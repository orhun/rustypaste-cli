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
/// Upload handler.
pub mod upload;

use crate::args::Args;
use crate::config::Config;
use crate::error::Result;
use crate::upload::Uploader;
use std::fs;
use std::io::{self, Read};

/// Runs `rustypaste-cli`.
pub fn run(args: Args) -> Result<()> {
    let mut config = if let Some(ref config_path) = args.config {
        toml::from_str(&fs::read_to_string(&config_path)?)?
    } else if let Some(path) = dirs_next::home_dir()
        .map(|p| p.join(".rustypaste").join("config.toml"))
        .map(|p| p.exists().then(|| p))
        .flatten()
    {
        toml::from_str(&fs::read_to_string(&path)?)?
    } else {
        Config::default()
    };
    config.update_from_args(&args);

    let uploader = Uploader::new(&config);
    if let Some(url) = args.url {
        match uploader.upload_url(&url) {
            Ok(short_url) => println!("{:?} -> {}", url, short_url.trim()),
            Err(e) => eprintln!("{:?} -> {}", url, e),
        }
    } else if args.files.contains(&String::from("-")) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        match uploader.upload_stream(buffer.as_bytes()) {
            Ok(url) => println!("stdin -> {}", url.trim()),
            Err(e) => eprintln!("stdin -> {}", e),
        }
    } else {
        for file in args.files {
            match uploader.upload_file(&file) {
                Ok(url) => println!("{:?} -> {}", file, url.trim()),
                Err(e) => eprintln!("{:?} -> {}", file, e),
            }
        }
    }

    Ok(())
}
