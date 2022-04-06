use anyhow::{bail, Context};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use you_must_conform::CheckItem;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "conform.yaml")]
    config: PathBuf,
    #[clap(short, long, default_value = ".")]
    folder: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    config: Vec<CheckItem>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = File::open(&args.config).context(format!(
        "Couldn't open config file {}",
        args.config.display()
    ))?;
    let config: Config = serde_yaml::from_reader(config).context("Couldn't parse config")?;
    let problems = you_must_conform::check_items(args.folder, config.config)
        .context("Unable to complete checking")?;
    match problems.len() {
        0 => Ok(()),
        n => {
            for problem in problems {
                eprintln!("{problem}");
            }
            bail!("Found {n} problems")
        }
    }
}
