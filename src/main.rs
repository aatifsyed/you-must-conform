use anyhow::{bail, Context};
use clap::{ArgGroup, Parser};
use futures::future::try_join_all;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use url::Url;
use you_must_conform::CheckItem;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
#[clap(group(
            ArgGroup::new("config_file")
                .required(true)
                .args(&["file", "url"]),
        ))]
struct Args {
    #[clap(
        short,
        long,
        default_value = "conform.yaml",
        help = "The config file to check"
    )]
    file: PathBuf,
    #[clap(
        short,
        long,
        default_value = ".",
        help = "The folder to check against the config file"
    )]
    context: PathBuf,
    #[clap(short, long, help = "A url to fetch the config file from instead")]
    url: Option<Url>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    config: Vec<CheckItem>,
    #[serde(default)]
    include: Vec<Url>,
}

impl Config {
    #[async_recursion::async_recursion]
    async fn resolve(self) -> anyhow::Result<Vec<CheckItem>> {
        let Self {
            mut config,
            include,
        } = self;

        let resolve_includes = include.into_iter().map(|url| async move {
            let response = reqwest::get(url.clone())
                .await
                .and_then(Response::error_for_status)
                .context(format!("Couldn't GET {url}"))?;
            let text = response
                .text()
                .await
                .context(format!("Couldn't decode response from {url}"))?;
            let config: Config = serde_yaml::from_str(&text)
                .context(format!("Couldn't serialize config from {url}"))?;
            anyhow::Ok(config.resolve().await.context("Nested resolution failed")?)
        });

        let includes = try_join_all(resolve_includes)
            .await
            .context("Error resolving includes")?
            .into_iter()
            .flatten();

        config.extend(includes);

        Ok(config)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = match args.url {
        Some(url) => Config {
            config: vec![],
            include: vec![url],
        },
        None => {
            let file = File::open(&args.file)
                .context(format!("Couldn't open config file {}", args.file.display()))?;
            serde_yaml::from_reader(file).context("Couldn't parse config")?
        }
    };
    let items = config.resolve().await?;
    let problems = you_must_conform::check_items(args.context, items)
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
