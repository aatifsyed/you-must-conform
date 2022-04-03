use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
mod json;
use generic_new::GenericNew;

pub fn check_file(
    path: impl AsRef<Path>,
    specs: impl IntoIterator<Item = FileSpec>,
) -> anyhow::Result<Vec<Problem>> {
    use FileSpec::*;
    use Problem::*;
    let mut problems = Vec::new();
    for content in specs {
        let path = path.as_ref().to_owned();
        match content {
            HasLength(expected) => {
                let actual = path
                    .metadata()
                    .context(format!("Couldn't get metadata of {}", path.display()))?
                    .len();
                problems.push(IncorrectLength {
                    path,
                    expected,
                    actual,
                })
            }
            MatchesRegex(regex) => {
                let s = fs::read_to_string(&path)
                    .context(format!("Couldn't read {}", path.display()))?;
                if !regex.is_match(&s) {
                    problems.push(RegexNotMatched { path, regex })
                }
            }
            Json(schema) => {
                let f = File::open(&path).context(format!("Couldn't open {}", path.display()))?;
                match serde_json::from_reader::<_, serde_json::Value>(f) {
                    Ok(value) => todo!(),
                    Err(err) => problems.push(InvalidFormat {
                        path,
                        format: "json",
                        err: err.into(),
                    }),
                }
            }
            Toml(schema) => todo!(),
            Yaml(schema) => todo!(),
        }
    }
    Ok(problems)
}

pub fn check_folder(
    path: impl AsRef<Path>,
    children: impl IntoIterator<Item = FilesAndFolders>,
) -> anyhow::Result<Vec<Problem>> {
    use FilesAndFolders::*;
    use Problem::*;
    let path = PathBuf::from(path.as_ref());
    let mut problems = Vec::new();
    for child in children {
        match child {
            File(file) => {
                let path = path.join(file.name);
                match path.is_file() {
                    true => problems.extend(check_file(path, file.specs)?),
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
                    true => problems.extend(check_folder(path, folder.children)?),
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
    Ok(problems)
}

#[derive(Debug, thiserror::Error)]
pub enum Problem {
    #[error("File {} has length {actual}, expected {expected}", .path.display())]
    IncorrectLength {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
    #[error("File {} couldn't be read in as {format}: {err:?}", .path.display())]
    InvalidFormat {
        path: PathBuf,
        format: &'static str,
        err: anyhow::Error,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Schema {
    Like(serde_json::Value),
    Schema(serde_json::Value),
}

#[derive(Debug, Clone)]
pub enum FileSpec {
    HasLength(u64),
    MatchesRegex(Regex),
    Json(Schema),
    Toml(Schema),
    Yaml(Schema),
}

#[derive(Debug, Clone, GenericNew)]
pub struct FilePresent {
    name: String,
    specs: Vec<FileSpec>,
}

#[derive(Debug, Clone, GenericNew)]
pub struct FileNotPresent {
    name: String,
}

#[derive(Debug, Clone, GenericNew)]
pub struct FolderPresent {
    name: String,
    children: Vec<FilesAndFolders>,
}

#[derive(Debug, Clone, GenericNew)]
pub struct FolderNotPresent {
    name: String,
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
        let problems = check_folder(d, [])?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));
        Ok(())
    }

    #[test]
    fn file() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::File::create(d.path().join("foo"))?;

        let problems = check_folder(&d, [FilePresent::new("foo", []).into()])?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(&d, [FileNotPresent::new("foo").into()])?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [DisallowedFile(_)]));

        Ok(())
    }

    #[test]
    fn folder() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::create_dir(d.path().join("foo"))?;

        let problems = check_folder(&d, [FolderPresent::new("foo", []).into()])?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(&d, [FolderNotPresent::new("foo").into()])?;
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
            [FolderPresent::new("foo", [FolderPresent::new("bar", []).into()]).into()],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_folder(
            &d,
            [FolderPresent::new("foo", [FolderNotPresent::new("bar").into()]).into()],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [DisallowedFolder(_)]));

        Ok(())
    }
}
