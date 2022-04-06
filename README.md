# you-must-conform

<div align="center">

[![crates-io](https://img.shields.io/crates/v/you-must-conform.svg)](https://crates.io/crates/you-must-conform)
[![docs-rs](https://docs.rs/you-must-conform/badge.svg)](https://docs.rs/you-must-conform)
[![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/you-must-conform)

</div>

A command-line tool for enforcing YAML|JSON|TOML|text file contents.

## Usage
```yaml
# conform.yaml
config:
- file: Cargo.toml
  format: toml
  schema:                   # Ensure these nested keys are set
    package:
      edition: "2021"
- file: Cargo.lock
  exists: true              # Ensure this file exists
- file: src/lib.rs
  matches-regex: '(?m)^use' # Ensure this regex is matched in the file

include:                    # (Recursively) merge config from these urls
- https://example.com/another-conform.yaml

```

```console
$ you-must-conform --help
you-must-conform 1.1.0
A command-line tool for enforcing YAML|JSON|TOML|text file contents.

USAGE:
    you-must-conform [OPTIONS] <--file <FILE>|--url <URL>>

OPTIONS:
    -c, --context <CONTEXT>    The folder to check against the config file [default: .]
    -f, --file <FILE>          The config file to check [default: conform.yaml]
    -h, --help                 Print help information
    -u, --url <URL>            A url to fetch the config file from instead
    -V, --version              Print version information

$ you-must-conform
Schema not matched in ./Cargo.toml:
    "package" is a required property
File ./Cargo.lock does not exist
File ./src/lib.rs does not match regex (?m)^use
Error: Found 3 problems
```

License: MIT
