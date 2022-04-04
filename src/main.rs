use anyhow::Context;
use clap::Parser;
use std::{fs::File, path::PathBuf};
use you_must_conform::{check_folder, FilesAndFolders};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    config: PathBuf,
    folder: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Args::parse();
    let config = File::open(opt.config).context("Couldn't open config file")?;
    let config: Vec<FilesAndFolders> =
        serde_yaml::from_reader(config).context("Invalid config file")?;
    let problems = check_folder(opt.folder, config).context("Couldn't check folder")?;
    println!("{problems:?}");
    Ok(())
}
