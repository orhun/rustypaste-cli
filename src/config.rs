use crate::args::Args;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::fs;

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
    /// Token for authentication, omitted when serializing the configuration.
    #[serde(skip_serializing)]
    pub auth_token: Option<SecretString>,
    /// A file containing the token for authentication.
    ///
    /// Leading and trailing whitespace will be trimmed.
    pub auth_token_file: Option<String>,
    /// Token for deleting files, omitted when serializing the configuration.
    #[serde(skip_serializing)]
    pub delete_token: Option<SecretString>,
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
                Ok(token) => self.server.auth_token = Some(token.trim().into()),
                Err(e) => eprintln!("Error while reading token file: {e}"),
            };
        };

        if let Some(path) = &self.server.delete_token_file {
            let path = shellexpand::tilde(path).to_string();
            match fs::read_to_string(path) {
                Ok(token) => self.server.delete_token = Some(token.trim().into()),
                Err(e) => eprintln!("Error while reading token file: {e}"),
            };
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn debug_masks_and_serialization_omits_tokens() {
        let cfg: Config = toml::from_str(
            r#"
                [server]
                address = "https://paste.example.com"
                auth_token = "auth-secret"
                delete_token = "delete-secret"

                [paste]
            "#,
        )
        .expect("configuration should deserialize");

        let debug = format!("{cfg:?}");
        let serialized = toml::to_string(&cfg).expect("configuration should serialize");

        for token in ["auth-secret", "delete-secret"] {
            assert!(!debug.contains(token));
            assert!(!serialized.contains(token));
        }
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    /// Test that the token file is being properly processed.
    fn test_parse_token_files_no_whitespace() {
        let mut cfg = Config::default();
        let token = "KBRRHMxlJfFo1".to_string();

        cfg.server.auth_token_file = Some("tests/token_file_parsing/token.txt".to_string());
        cfg.parse_token_files();
        assert_eq!(
            cfg.server
                .auth_token
                .as_ref()
                .map(|token| token.expose_secret()),
            Some(token.as_str())
        );
    }

    #[test]
    /// Test that whitespace is being properly trimmed.
    fn test_parse_token_files_whitespaced() {
        let mut cfg = Config::default();
        let token = "nhJuLuY5vxUrO".to_string();

        cfg.server.auth_token_file =
            Some("tests/token_file_parsing/token_whitespaced.txt".to_string());
        cfg.parse_token_files();
        assert_eq!(
            cfg.server
                .auth_token
                .as_ref()
                .map(|token| token.expose_secret()),
            Some(token.as_str())
        );
    }
}
