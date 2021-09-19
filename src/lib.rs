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
use colored::Colorize;
use std::fs;
use std::io::{self, Read};

/// Runs `rpaste`.
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

    let mut results = Vec::new();
    let uploader = Uploader::new(&config);
    if let Some(ref url) = args.url {
        results.push(uploader.upload_url(url));
    } else if args.files.contains(&String::from("-")) {
        let mut buffer = Vec::new();
        let stdin = io::stdin();
        for bytes in stdin.bytes() {
            buffer.push(bytes?);
        }
        results.push(uploader.upload_stream(&*buffer));
    } else {
        for file in args.files.iter() {
            results.push(uploader.upload_file(file))
        }
    }

    let format_padding = args
        .prettify
        .then(|| results.iter().map(|v| v.0.len()).max())
        .flatten()
        .unwrap_or(1);
    for (data, result) in results.iter().map(|v| (v.0, v.1.as_ref())) {
        let data = if args.prettify {
            format!(
                "{:p$} {} ",
                data,
                if result.is_ok() {
                    "=>".green().bold()
                } else {
                    "=>".red().bold()
                },
                p = format_padding,
            )
        } else {
            String::new()
        };
        match result {
            Ok(url) => println!("{}{}", data, url.trim()),
            Err(e) => eprintln!("{}{}", data, e),
        }
    }

    Ok(())
}
