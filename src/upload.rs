use crate::config::ServerConfig;
use crate::error::{Error, Result};
use multipart::client::lazy::Multipart;
use std::io::Read;
use ureq::{Agent, AgentBuilder};

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// Upload handler.
#[derive(Debug)]
pub struct Uploader<'a> {
    /// HTTP client.
    client: Agent,
    /// Server configuration.
    config: &'a ServerConfig,
}

impl<'a> Uploader<'a> {
    /// Constructs a new instance.
    pub fn new(config: &'a ServerConfig) -> Self {
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
        let mut multipart = Multipart::new();
        multipart.add_file("file", file);

        self.upload(multipart)
    }

    /// Uploads a stream to the server.
    pub fn upload_stream<S: Read>(&self, stream: S) -> Result<String> {
        let mut multipart = Multipart::new();
        multipart.add_stream("file", stream, DEFAULT_FILE_NAME, None);

        self.upload(multipart)
    }

    /// Uploads the given multipart data.
    fn upload(&self, mut multipart: Multipart<'static, '_>) -> Result<String> {
        let multipart_data = multipart.prepare()?;
        let mut request = self.client.post(&self.config.address).set(
            "Content-Type",
            &format!(
                "multipart/form-data; boundary={}",
                multipart_data.boundary()
            ),
        );
        if let Some(auth_token) = &self.config.auth_token {
            request = request.set("Authorization", auth_token);
        }
        let response = request
            .send(multipart_data)
            .map_err(|e| Error::RequestError(Box::new(e)))?;
        Ok(response.into_string()?)
    }
}
