use crate::config::Config;
use crate::error::{Error, Result};
use multipart::client::lazy::Multipart;
use std::io::Read;
use ureq::{Agent, AgentBuilder};

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// HTTP header to use for specifying expiration times.
const EXPIRATION_HEADER: &str = "expire";

/// Upload handler.
#[derive(Debug)]
pub struct Uploader<'a> {
    /// HTTP client.
    client: Agent,
    /// Server configuration.
    config: &'a Config,
}

impl<'a> Uploader<'a> {
    /// Constructs a new instance.
    pub fn new(config: &'a Config) -> Self {
        Self {
            client: AgentBuilder::new()
                .user_agent(&format!(
                    "{}/{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ))
                .build(),
            config,
        }
    }

    /// Uploads the given file to the server.
    pub fn upload_file(&self, file: &str) -> Result<String> {
        let field = if self.config.paste.oneshot == Some(true) {
            "oneshot"
        } else {
            "file"
        };
        let mut multipart = Multipart::new();
        multipart.add_file(field, file);

        self.upload(multipart)
    }

    /// Uploads the given url (stream) to the server.
    pub fn upload_url(&self, url: &str) -> Result<String> {
        let mut multipart = Multipart::new();
        multipart.add_stream::<_, &[u8], &str>("url", url.as_bytes(), None, None);

        self.upload(multipart)
    }

    /// Uploads a stream to the server.
    pub fn upload_stream<S: Read>(&self, stream: S) -> Result<String> {
        let field = if self.config.paste.oneshot == Some(true) {
            "oneshot"
        } else {
            "file"
        };
        let mut multipart = Multipart::new();
        multipart.add_stream(field, stream, DEFAULT_FILE_NAME, None);

        self.upload(multipart)
    }

    /// Uploads the given multipart data.
    fn upload(&self, mut multipart: Multipart<'static, '_>) -> Result<String> {
        let multipart_data = multipart.prepare()?;
        let mut request = self.client.post(&self.config.server.address).set(
            "Content-Type",
            &format!(
                "multipart/form-data; boundary={}",
                multipart_data.boundary()
            ),
        );
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.set("Authorization", auth_token);
        }
        if let Some(expiration_time) = &self.config.paste.expire {
            request = request.set(EXPIRATION_HEADER, expiration_time);
        }
        let response = request
            .send(multipart_data)
            .map_err(|e| Error::RequestError(Box::new(e)))?;
        Ok(response.into_string()?)
    }
}
