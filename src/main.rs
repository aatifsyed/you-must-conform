use anyhow::Context;
use clap::Parser;
use generic_new::GenericNew;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};
use you_must_conform::{check_folder, FilesAndFolders};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    config: PathBuf,
    folder: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, GenericNew)]
struct YouMustConformConfig {
    config: Vec<FilesAndFolders>,
}

fn main() -> anyhow::Result<()> {
    let opt = Args::parse();
    let config = File::open(opt.config).context("Couldn't open config file")?;
    let config: YouMustConformConfig =
        serde_yaml::from_reader(config).context("Invalid config file")?;
    let problems = check_folder(opt.folder, config.config).context("Couldn't check folder")?;
    println!("{problems:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use you_must_conform::{FileFormat::Toml, FilePresent, MatchesSchema, Schema::Infer};

    use crate::YouMustConformConfig;

    #[test]
    fn dump_config() -> anyhow::Result<()> {
        let config = YouMustConformConfig::new([FilePresent::new(
            "Cargo.toml",
            [MatchesSchema::new(
                Toml,
                Infer(json!(
                    {"package":{"edition": "2021"}}
                )),
            )
            .into()],
        )
        .into()]);
        let config = serde_yaml::to_string(&config)?;
        println!("{config}");
        Ok(())
    }
}
