//! A CLI tool for [`rustypaste`].
//!
//! [`rustypaste`]: https://github.com/orhun/rustypaste
#![warn(missing_docs, clippy::unwrap_used)]

/// Command-line argument parser.
pub mod args;
/// Configuration file parser.
pub mod config;
/// Custom error implementation.
pub mod error;
/// Upload helper.
pub mod upload;

use crate::args::Args;
use crate::config::Config;
use crate::error::Result;
use std::fs;

/// Runs `rustypaste-cli`.
pub fn run(mut args: Args) -> Result<()> {
    let mut config = Config::default();

    if let Some(config_dir) = dirs_next::home_dir() {
        let path = config_dir.join(".rustypaste").join("config.toml");
        if path.exists() {
            args.config = Some(path);
        }
    }
    if let Some(ref config_path) = args.config {
        config = toml::from_str::<Config>(&fs::read_to_string(&config_path)?)?;
    }
    config.update_from_args(&args);

    for file in args.files {
        match upload::upload_file(&file, &config.server) {
            Ok(url) => println!("{:?} -> {}", file, url.trim()),
            Err(e) => eprintln!("{:?} -> {}", file, e),
        }
    }

    Ok(())
}
