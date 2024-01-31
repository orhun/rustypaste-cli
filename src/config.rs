use crate::args::Args;
use serde::{Deserialize, Serialize};

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
    /// Token for deleting files.
    pub delete_token: Option<String>,
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
}
