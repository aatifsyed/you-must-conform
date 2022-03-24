use regex::Regex;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
mod json;

pub fn check_file(path: impl AsRef<Path>, specs: Vec<FileSpec>) -> Vec<Problem> {
    use FileSpec::*;
    use Problem::*;
    let mut problems = Vec::new();
    for content in specs {
        let path = path.as_ref().to_owned();
        match content {
            HasLength(expected) => match path.metadata() {
                Ok(metadata) => {
                    let actual = metadata.len();
                    problems.push(IncorrectLength {
                        path,
                        expected,
                        actual,
                    })
                }
                Err(err) => problems.push(IoProblem {
                    path,
                    err,
                    operation: "Reading length",
                }),
            },
            ContainsRegex(regex) => match fs::read_to_string(&path) {
                Ok(s) => {
                    if !regex.is_match(&s) {
                        problems.push(RegexNotMatched { path, regex })
                    }
                }
                Err(err) => problems.push(IoProblem {
                    path,
                    err,
                    operation: "Matching regex",
                }),
            },
        }
    }
    problems
}

pub fn check_folder(path: impl AsRef<Path>, children: Vec<FilesAndFolders>) -> Vec<Problem> {
    use FilesAndFolders::*;
    use Problem::*;
    let path = PathBuf::from(path.as_ref());
    let mut problems = Vec::new();
    for child in children {
        match child {
            File(file) => {
                let path = path.join(file.name);
                match path.is_file() {
                    true => problems.extend(check_file(path, file.specs)),
                    false => problems.push(FileNotPresent(path)),
                }
            }
            NotFile(not_file) => {
                let path = path.join(not_file.name);
                if path.is_file() {
                    problems.push(DisallowedFile(path))
                }
            }
            Folder(folder) => {
                let path = path.join(folder.name);
                match path.is_dir() {
                    true => problems.extend(check_folder(path, folder.children)),
                    false => problems.push(FolderNotPresent(path)),
                }
            }
            NotFolder(not_folder) => {
                let path = path.join(not_folder.name);
                if path.is_dir() {
                    problems.push(DisallowedFolder(path))
                }
            }
        }
    }
    problems
}

#[derive(Debug, thiserror::Error)]
pub enum Problem {
    #[error("File {} has length {actual}, expected {expected}", .path.display())]
    IncorrectLength {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
    #[error("Error accessing {} for {operation}: {err}", .path.display())]
    IoProblem {
        path: PathBuf,
        err: io::Error,
        operation: &'static str,
    },
    #[error("File {} does not match regex {regex}", .path.display())]
    RegexNotMatched { path: PathBuf, regex: Regex },
    #[error("File {} does not exist", .0.display())]
    FileNotPresent(PathBuf),
    #[error("File {} is not allowed to exist", .0.display())]
    DisallowedFile(PathBuf),
    #[error("Folder {} does not exist", .0.display())]
    FolderNotPresent(PathBuf),
    #[error("Folder {} is not allowed to exist", .0.display())]
    DisallowedFolder(PathBuf),
}

#[derive(Debug, Clone)]
pub enum FileSpec {
    HasLength(u64),
    ContainsRegex(Regex),
}

#[derive(Debug, Clone)]
pub struct FilePresent {
    name: String,
    specs: Vec<FileSpec>,
}

impl FilePresent {
    pub fn new(name: impl AsRef<str>, specs: impl AsRef<[FileSpec]>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            specs: specs.as_ref().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileNotPresent {
    name: String,
}

impl FileNotPresent {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct FolderPresent {
    name: String,
    children: Vec<FilesAndFolders>,
}

impl FolderPresent {
    pub fn new(name: impl AsRef<str>, children: &[FilesAndFolders]) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            children: children.as_ref().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderNotPresent {
    name: String,
}

impl FolderNotPresent {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
        }
    }
}

#[derive(Debug, derive_more::From, Clone)]
pub enum FilesAndFolders {
    File(FilePresent),
    NotFile(FileNotPresent),
    Folder(FolderPresent),
    NotFolder(FolderNotPresent),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::{
        check_folder, FileNotPresent, FilePresent, FolderNotPresent, FolderPresent,
        Problem::{DisallowedFile, DisallowedFolder},
    };

    #[test]
    fn empty_dir() -> anyhow::Result<()> {
        let d = tempdir()?;
        let problems = check_folder(d, vec![]);
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));
        Ok(())
    }

    #[test]
    fn file() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::File::create(d.path().join("foo"))?;

        let problems = check_folder(&d, vec![FilePresent::new("foo", []).into()]);
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(&d, vec![FileNotPresent::new("foo").into()]);
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [DisallowedFile(_)]));

        Ok(())
    }

    #[test]
    fn folder() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::create_dir(d.path().join("foo"))?;

        let problems = check_folder(&d, vec![FolderPresent::new("foo", &[]).into()]);
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(&d, vec![FolderNotPresent::new("foo").into()]);
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [DisallowedFolder(_)]));

        Ok(())
    }

    #[test]
    fn nested_folder() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::create_dir_all(d.path().join("foo").join("bar"))?;

        let problems = check_folder(
            &d,
            vec![FolderPresent::new("foo", &[FolderPresent::new("bar", &[]).into()]).into()],
        );
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(
            &d,
            vec![FolderPresent::new("foo", &[FolderNotPresent::new("bar").into()]).into()],
        );
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [DisallowedFolder(_)]));

        Ok(())
    }
}
