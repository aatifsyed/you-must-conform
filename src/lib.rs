//! <div align="center">
//!
//! [![crates-io](https://img.shields.io/crates/v/you-must-conform.svg)](https://crates.io/crates/you-must-conform)
//! [![docs-rs](https://docs.rs/you-must-conform/badge.svg)](https://docs.rs/you-must-conform)
//! [![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/you-must-conform)
//!
//! </div>
//!
//! A command-line tool for enforcing YAML|JSON|TOML|text file contents.
//!
//! # Usage
//! ```yaml
//! # conform.yaml
//! config:
//! - file: Cargo.toml
//!   format: toml
//!   schema:                   # Ensure these nested keys are set
//!     package:
//!       edition: "2021"
//! - file: Cargo.lock
//!   exists: true              # Ensure this file exists
//! - file: src/lib.rs
//!   matches-regex: '(?m)^use' # Ensure this regex is matched in the file
//!
//! include:                    # (Recursively) merge config from these urls
//! - https://example.com/another-conform.yaml
//!
//! ```
//!
//! ```console
//! $ you-must-conform --help
//! you-must-conform 1.1.0
//! A command-line tool for enforcing YAML|JSON|TOML|text file contents.
//!
//! USAGE:
//!     you-must-conform [OPTIONS] <--file <FILE>|--url <URL>>
//!
//! OPTIONS:
//!     -c, --context <CONTEXT>    The folder to check against the config file [default: .]
//!     -f, --file <FILE>          The config file to check [default: conform.yaml]
//!     -h, --help                 Print help information
//!     -u, --url <URL>            A url to fetch the config file from instead
//!     -V, --version              Print version information
//!
//! $ you-must-conform
//! Schema not matched in ./Cargo.toml:
//!     "package" is a required property
//! File ./Cargo.lock does not exist
//! File ./src/lib.rs does not match regex (?m)^use
//! Error: Found 3 problems
//! ```

use anyhow::Context;
use itertools::Itertools;
use jsonschema::{JSONSchema, ValidationError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};
mod json;

use crate::json::describe_value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CheckItem {
    File {
        file: PathBuf,
        #[serde(flatten)]
        check: FileCheck,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum FileCheck {
    Exists {
        exists: bool,
    },
    LooksLike {
        format: FileFormat,
        schema: serde_json::Value,
    },
    #[serde(rename_all = "kebab-case")]
    MatchesRegex {
        #[serde(with = "serde_regex")]
        matches_regex: Regex,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, strum::IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
pub enum FileFormat {
    Json,
    Toml,
    Yaml,
}

pub fn check_items(
    root: impl AsRef<Path>,
    items: impl IntoIterator<Item = CheckItem>,
) -> anyhow::Result<Vec<Problem>> {
    use Problem::{
        DisallowedFile, FileNotPresent, InvalidFormat, RegexNotMatched, SchemaNotMatched,
    };
    let mut problems = Vec::new();
    let root = root.as_ref().to_owned();
    for item in items {
        match item {
            CheckItem::File { file, check } => {
                let path = root.join(file);
                match check {
                    FileCheck::Exists {
                        exists: should_exist,
                    } => match path.metadata() {
                        Ok(meta) if meta.is_file() && !should_exist => {
                            problems.push(DisallowedFile(path))
                        }
                        Err(err) if err.kind() == io::ErrorKind::NotFound && should_exist => {
                            problems.push(FileNotPresent(path))
                        }
                        _ => (),
                    },
                    FileCheck::LooksLike {
                        format,
                        schema: like,
                    } => match path.is_file() {
                        true => {
                            // Read to string since `toml` doesn't have a from_reader
                            let s = fs::read_to_string(&path)
                                .context(format!("Couldn't read {}", path.display()))?;
                            let deser_result = match format {
                                FileFormat::Json => {
                                    serde_json::from_str(&s).map_err(anyhow::Error::new)
                                }
                                FileFormat::Toml => toml::from_str(&s).map_err(anyhow::Error::new),
                                FileFormat::Yaml => {
                                    serde_yaml::from_str(&s).map_err(anyhow::Error::new)
                                }
                            };
                            match deser_result {
                                Ok(v) => {
                                    let schema = JSONSchema::compile(&describe_value(&like))
                                        .expect("Autogenerated schema generation failed, please file a bug report.");

                                    if let Err(errors) = schema.validate(&v) {
                                        problems.push(SchemaNotMatched {
                                            path,
                                            errors: errors
                                                .map(|validation_error| ValidationError {
                                                    instance: Cow::Owned(
                                                        validation_error.instance.into_owned(),
                                                    ),
                                                    ..validation_error
                                                })
                                                .collect(),
                                        })
                                    };
                                }
                                Err(err) => problems.push(InvalidFormat {
                                    path,
                                    format: format.into(),
                                    err,
                                }),
                            }
                        }
                        false => problems.push(FileNotPresent(path)),
                    },
                    FileCheck::MatchesRegex {
                        matches_regex: regex,
                    } => match path.is_file() {
                        true => {
                            let s = fs::read_to_string(&path)
                                .context(format!("Couldn't read {}", path.display()))?;
                            if !regex.is_match(&s) {
                                problems.push(RegexNotMatched { path, regex })
                            }
                        }
                        false => problems.push(FileNotPresent(path)),
                    },
                }
            }
        }
    }
    Ok(problems)
}

#[derive(Debug, thiserror::Error)]
pub enum Problem {
    #[error("File {} couldn't be read in as {format}: {err:?}", .path.display())]
    InvalidFormat {
        path: PathBuf,
        format: &'static str,
        err: anyhow::Error,
    },
    #[error("Schema not matched in {}:\n\t{}", .path.display(), .errors.iter().join("\n\t"))]
    SchemaNotMatched {
        path: PathBuf,
        errors: Vec<ValidationError<'static>>,
    },
    #[error("File {} does not match regex {regex}", .path.display())]
    RegexNotMatched { path: PathBuf, regex: Regex },
    #[error("File {} does not exist", .0.display())]
    FileNotPresent(PathBuf),
    #[error("File {} is not allowed to exist", .0.display())]
    DisallowedFile(PathBuf),
}

impl CheckItem {
    pub fn file(file: impl AsRef<Path>, check: FileCheck) -> Self {
        Self::File {
            file: file.as_ref().to_owned(),
            check,
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use serde_json::json;
    use std::fs::{self, File};
    use tempfile::tempdir;

    use crate::{check_items, CheckItem, FileCheck, FileFormat, Problem};

    #[test]
    fn empty_directory() -> anyhow::Result<()> {
        let d = tempdir()?;
        let problems = check_items(d, [])?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));
        Ok(())
    }

    #[test]
    fn file_existence() -> anyhow::Result<()> {
        let d = tempdir()?;
        File::create(d.path().join("foo"))?;

        let problems = check_items(
            &d,
            [CheckItem::file("foo", FileCheck::Exists { exists: true })],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_items(
            &d,
            [CheckItem::file("foo", FileCheck::Exists { exists: false })],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), [Problem::DisallowedFile(_)]));

        Ok(())
    }

    #[test]
    fn schema_validation() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::write(d.path().join("foo.toml"), "[hello]\nworld = true")?;
        let problems = check_items(
            &d,
            [CheckItem::file(
                "foo.toml",
                FileCheck::LooksLike {
                    format: FileFormat::Toml,
                    schema: json!({"hello": {"world": true}}),
                },
            )],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_items(
            &d,
            [CheckItem::file(
                "foo.toml",
                FileCheck::LooksLike {
                    format: FileFormat::Toml,
                    schema: json!({"hello": {"world": false}}),
                },
            )],
        )?;
        println!("{problems:?}");
        assert!(matches!(
            problems.as_slice(),
            [Problem::SchemaNotMatched { .. }]
        ));

        Ok(())
    }

    #[test]
    fn regex_matching() -> anyhow::Result<()> {
        let d = tempdir()?;
        fs::write(d.path().join("bar"), "barometer\nbartholomew\nbartender")?;
        let problems = check_items(
            &d,
            [CheckItem::file(
                "bar",
                FileCheck::MatchesRegex {
                    matches_regex: Regex::new("barth")?,
                },
            )],
        )?;
        println!("{problems:?}");
        assert!(matches!(problems.as_slice(), []));

        let problems = check_items(
            &d,
            [CheckItem::file(
                "bar",
                FileCheck::MatchesRegex {
                    matches_regex: Regex::new("foo")?,
                },
            )],
        )?;
        println!("{problems:?}");
        assert!(matches!(
            problems.as_slice(),
            [Problem::RegexNotMatched { .. }]
        ));

        Ok(())
    }
}
