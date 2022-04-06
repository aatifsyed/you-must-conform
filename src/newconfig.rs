use generic_new::GenericNew;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, GenericNew)]
struct Config {
    config: Vec<NodeSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum NodeSpec {
    FileSpec(FileSpec),
}

#[derive(Debug, Serialize, Deserialize, GenericNew)]
struct FileSpec {
    file: PathBuf,
    #[serde(flatten)]
    check: FileCheck,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FileCheck {
    Exists(bool),
    MatchesSchema(serde_json::Value),
}

#[test]
fn dump_config() -> anyhow::Result<()> {
    let config = Config::new([NodeSpec::FileSpec(FileSpec::new(
        "./Cargo.toml",
        FileCheck::MatchesSchema(json!({"package": {"edition": "2021"}})),
    ))]);
    println!("{}", serde_yaml::to_string(&config)?);
    Ok(())
}
