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
use crate::error::{Error, Result};
use crate::upload::Uploader;
use colored::Colorize;
use std::fs;
use std::io::{self, Read};

/// Default name of the configuration file.
const CONFIG_FILE: &str = "config.toml";

/// Runs `rpaste`.
pub fn run(args: Args) -> Result<()> {
    let mut config = Config::default();
    if let Some(ref config_path) = args.config {
        config = toml::from_str(&fs::read_to_string(config_path)?)?
    } else {
        for path in [
            dirs_next::home_dir().map(|p| p.join(".rustypaste").join(CONFIG_FILE)),
            dirs_next::config_dir().map(|p| p.join("rustypaste").join(CONFIG_FILE)),
        ]
        .iter()
        .filter_map(|v| v.as_ref())
        {
            if path.exists() {
                config = toml::from_str(&fs::read_to_string(path)?)?;
                break;
            }
        }
    }
    config.update_from_args(&args);
    if config.server.address.is_empty() {
        return Err(Error::NoServerAddressError);
    }

    let uploader = Uploader::new(&config);
    if args.print_server_version {
        println!("rustypaste-server {}", uploader.retrieve_version()?);
        return Ok(());
    }

    let mut results = Vec::new();
    if let Some(ref url) = args.url {
        results.push(uploader.upload_url(url));
    } else if let Some(ref remote_url) = args.remote {
        results.push(uploader.upload_remote_url(remote_url));
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
    let prettify = args.prettify
        || config
            .style
            .as_ref()
            .map(|style| style.prettify)
            .unwrap_or(false);
    let format_padding = prettify
        .then(|| results.iter().map(|v| v.0.len()).max())
        .flatten()
        .unwrap_or(1);
    for (data, result) in results.iter().map(|v| (v.0, v.1.as_ref())) {
        let data = if prettify {
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
            Err(e) => eprintln!("{data}{e}"),
        }
    }

    Ok(())
}
