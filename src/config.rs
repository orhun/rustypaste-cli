use crate::args::Args;
use serde::{Deserialize, Serialize};

/// Configuration values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,
    /// Paste configuration.
    pub paste: PasteConfig,
}

/// Server configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server address.
    pub address: String,
    /// Token for authentication.
    pub auth_token: Option<String>,
}

/// Paste configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PasteConfig {
    /// Whether if the file will disappear after being viewed once.
    pub oneshot: Option<bool>,
}

impl Config {
    /// Override the configuration file with arguments.
    pub fn update_from_args(&mut self, args: &Args) {
        if let Some(server_address) = &args.server {
            self.server.address = server_address.to_string();
        }
        if args.auth.is_some() {
            self.server.auth_token = args.auth.as_ref().cloned();
        }
        if args.oneshot {
            self.paste.oneshot = Some(true);
        }
    }
}
