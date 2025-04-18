use crate::args::Args;
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(target_os = "macos")]
use std::{env, path::PathBuf};

/// Configuration values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,
    /// Paste configuration.
    pub paste: PasteConfig,
    /// Style configuration.
    pub style: Option<StyleConfig>,
}

/// Server configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server address.
    pub address: String,
    /// Token for authentication.
    pub auth_token: Option<String>,
    /// A file containing the token for authentication.
    ///
    /// Leading and trailing whitespace will be trimmed.
    pub auth_token_file: Option<String>,
    /// Token for deleting files.
    pub delete_token: Option<String>,
    /// A file containing the token for deleting files.
    ///
    /// Leading and trailing whitespace will be trimmed.
    pub delete_token_file: Option<String>,
}

/// Paste configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PasteConfig {
    /// Whether if the file will disappear after being viewed once.
    pub oneshot: Option<bool>,
    /// Expiration time for the link.
    pub expire: Option<String>,
    /// Filename.
    #[serde(skip_deserializing)]
    pub filename: Option<String>,
}

/// Style configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Whether if the output will be prettified.
    pub prettify: bool,
}

impl Config {
    /// Override the configuration file with arguments.
    pub fn update_from_args(&mut self, args: &Args) {
        if let Some(server_address) = &args.server {
            self.server.address = server_address.to_string();
        }
        if args.auth.is_some() {
            self.server.auth_token = args.auth.as_ref().cloned();
            if args.delete {
                self.server.delete_token = args.auth.as_ref().cloned();
            }
        }
        if args.oneshot {
            self.paste.oneshot = Some(true);
        }
        if args.expire.is_some() {
            self.paste.expire = args.expire.as_ref().cloned();
        }
        if args.filename.is_some() {
            self.paste.filename = args.filename.as_ref().cloned();
        }
    }

    /// Parses the files referenced by [Config::auth_token_file] and [Config::delete_token_file].
    ///
    /// Updates the respective token variables with the contents of the files.
    pub fn parse_token_files(&mut self) {
        if let Some(path) = &self.server.auth_token_file {
            let path = shellexpand::tilde(path).to_string();
            match fs::read_to_string(path) {
                Ok(token) => self.server.auth_token = Some(token.trim().to_string()),
                Err(e) => eprintln!("Error while reading token file: {e}"),
            };
        };

        if let Some(path) = &self.server.delete_token_file {
            let path = shellexpand::tilde(path).to_string();
            match fs::read_to_string(path) {
                Ok(token) => self.server.delete_token = Some(token.trim().to_string()),
                Err(e) => eprintln!("Error while reading token file: {e}"),
            };
        };
    }

    /// Find a special config path on macOS.
    /// 
    /// The `dirs-next` crate ignores the `XDG_CONFIG_HOME` env var on macOS and only considers
    /// `Library/Application Support` as the config dir, which is primarily used by GUI apps.
    ///
    /// This function determines the config path and honors the `XDG_CONFIG_HOME` env var.
    /// If it is not set, it will fall back to `~/.config`
    #[cfg(target_os = "macos")]
    pub(crate) fn retrieve_xdg_config_on_macos(&self) -> PathBuf {
        let config_dir = env::var("XDG_CONFIG_HOME").map_or_else(
            |_| dirs_next::home_dir().unwrap_or_default().join(".config"),
            PathBuf::from,
        );
        config_dir.join("rustypaste")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test that the token file is being properly processed.
    fn test_parse_token_files_no_whitespace() {
        let mut cfg = Config::default();
        let token = "KBRRHMxlJfFo1".to_string();

        cfg.server.auth_token_file = Some("tests/token_file_parsing/token.txt".to_string());
        cfg.parse_token_files();
        assert_eq!(cfg.server.auth_token, Some(token));
    }

    #[test]
    /// Test that whitespace is being properly trimmed.
    fn test_parse_token_files_whitespaced() {
        let mut cfg = Config::default();
        let token = "nhJuLuY5vxUrO".to_string();

        cfg.server.auth_token_file =
            Some("tests/token_file_parsing/token_whitespaced.txt".to_string());
        cfg.parse_token_files();
        assert_eq!(cfg.server.auth_token, Some(token));
    }
}
