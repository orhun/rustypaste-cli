use crate::config::Config;
use crate::error::{Error, Result};
use multipart::client::lazy::Multipart;
use std::io::Read;
use ureq::Error as UreqError;
use ureq::{Agent, AgentBuilder};

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// HTTP header to use for specifying expiration times.
const EXPIRATION_HEADER: &str = "expire";

/// Wrapper around raw data and result.
#[derive(Debug)]
pub struct UploadResult<'a, T>(pub &'a str, pub Result<T>);

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
    pub fn upload_file(&self, file: &'a str) -> UploadResult<'a, String> {
        let field = if self.config.paste.oneshot == Some(true) {
            "oneshot"
        } else {
            "file"
        };
        let mut multipart = Multipart::new();
        multipart.add_file(field, file);

        UploadResult(file, self.upload(multipart))
    }

    /// Uploads the given url (stream) to the server.
    pub fn upload_url(&self, url: &'a str) -> UploadResult<'a, String> {
        let mut multipart = Multipart::new();
        multipart.add_stream::<_, &[u8], &str>("url", url.as_bytes(), None, None);

        UploadResult(url, self.upload(multipart))
    }

    /// Uploads a stream to the server.
    pub fn upload_stream<S: Read>(&self, stream: S) -> UploadResult<'a, String> {
        let field = if self.config.paste.oneshot == Some(true) {
            "oneshot"
        } else {
            "file"
        };
        let mut multipart = Multipart::new();
        multipart.add_stream(field, stream, DEFAULT_FILE_NAME, None);

        UploadResult("stream", self.upload(multipart))
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
        match request.send(multipart_data) {
            Ok(response) => {
                let status = response.status();
                let response_text = response.into_string()?;
                if response_text.lines().count() != 1 {
                    Err(Error::UploadError(format!(
                        "server returned invalid body (status code: {})",
                        status
                    )))
                } else if status == 200 {
                    Ok(response_text)
                } else {
                    Err(Error::UploadError(format!(
                        "unknown error (status code: {})",
                        status
                    )))
                }
            }
            Err(UreqError::Status(code, response)) => Err(Error::UploadError(format!(
                "{} (status code: {})",
                response.into_string()?,
                code
            ))),
            Err(e) => Err(Error::RequestError(Box::new(e))),
        }
    }
}
