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
use etcetera::BaseStrategy;
use std::fs;
use std::io::IsTerminal;
use std::io::{self, Read};

/// Default name of the configuration file.
const CONFIG_FILE: &str = "config.toml";

/// Explicit file args win over an implicit non-TTY stdin, so `rpaste file`
/// under a Ctrl-Z-suspended TUI (inherited non-TTY pipe, no EOF) uploads the
/// file instead of blocking `read_to_end` forever or sending an empty file.
fn should_read_stdin(files: &[String], stdin_is_tty: bool) -> bool {
    files.iter().any(|f| f == "-") || (files.is_empty() && !stdin_is_tty)
}

/// Runs `rpaste`.
pub fn run(args: Args) -> Result<()> {
    let mut config = Config::default();
    if let Some(ref config_path) = args.config {
        config = toml::from_str(&fs::read_to_string(config_path)?)?
    } else {
        // cannot panic - see https://github.com/lunacookies/etcetera/issues/42
        let strategy = etcetera::choose_base_strategy()
            .expect("cannot determine current OS's default strategy (layout)");
        for path in [
            strategy.config_dir().join("rustypaste").join(CONFIG_FILE),
            // paths for backwards compatibility
            #[cfg(target_family = "unix")]
            strategy
                .home_dir()
                .to_path_buf()
                .join(".rustypaste")
                .join(CONFIG_FILE),
            #[cfg(target_os = "macos")]
            strategy
                .home_dir()
                .to_path_buf()
                .join("Library/Application Support/rustypaste")
                .join(CONFIG_FILE),
        ]
        .iter()
        {
            if path.exists() {
                config = toml::from_str(&fs::read_to_string(path)?)?;
                break;
            }
        }
    }
    config.parse_token_files();
    config.update_from_args(&args);
    if config.server.address.is_empty() {
        return Err(Error::NoServerAddressError);
    }

    let uploader = Uploader::new(&config);
    if args.print_server_version {
        println!("rustypaste-server {}", uploader.retrieve_version()?.trim());
        return Ok(());
    }

    if args.list_files {
        let prettify = args.prettify
            || config
                .style
                .as_ref()
                .map(|style| style.prettify)
                .unwrap_or(false);
        uploader.retrieve_list(&mut io::stdout(), prettify)?;
        return Ok(());
    }

    let mut results = Vec::new();
    if let Some(ref url) = args.url {
        results.push(uploader.upload_url(url));
    } else if let Some(ref remote_url) = args.remote {
        results.push(uploader.upload_remote_url(remote_url));
    } else if should_read_stdin(&args.files, std::io::stdin().is_terminal()) {
        let mut buffer = Vec::new();
        let mut stdin = io::stdin();
        stdin.read_to_end(&mut buffer)?;
        results.push(uploader.upload_stream(&*buffer));
    } else {
        for file in args.files.iter() {
            if !args.delete {
                results.push(uploader.upload_file(file))
            } else {
                results.push(uploader.delete_file(file))
            }
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

#[cfg(test)]
mod tests {
    use super::should_read_stdin;

    fn files(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn explicit_file_wins_over_non_tty_stdin() {
        assert!(!should_read_stdin(&files(&["f.html"]), false));
    }

    #[test]
    fn dash_sentinel_always_reads_stdin() {
        assert!(should_read_stdin(&files(&["-"]), true));
        assert!(should_read_stdin(&files(&["-"]), false));
    }

    #[test]
    fn bare_pipeline_reads_stdin() {
        assert!(should_read_stdin(&files(&[]), false));
    }

    #[test]
    fn interactive_no_files_does_not_read_stdin() {
        assert!(!should_read_stdin(&files(&[]), true));
    }
}
