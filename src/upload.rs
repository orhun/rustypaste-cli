use crate::config::Config;
use crate::error::{Error, Result};
use indicatif::{ProgressBar, ProgressStyle};
use multipart::client::lazy::Multipart;
use std::io::{Read, Result as IoResult};
use std::time::Duration;
use ureq::Error as UreqError;
use ureq::{Agent, AgentBuilder};
use url::Url;

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// HTTP header to use for specifying expiration times.
const EXPIRATION_HEADER: &str = "expire";

/// Wrapper around raw data and result.
#[derive(Debug)]
pub struct UploadResult<'a, T>(pub &'a str, pub Result<T>);

/// Upload progress tracker.
#[derive(Debug)]
pub struct UploadTracker<'a, R: Read> {
    /// Inner type for the upload stream.
    inner: R,
    /// Progress bar.
    progress_bar: &'a ProgressBar,
    /// Uploaded size.
    uploaded: usize,
}

impl<'a, R: Read> UploadTracker<'a, R> {
    /// Constructs a new instance.
    pub fn new(progress_bar: &'a ProgressBar, total: u64, reader: R) -> Result<Self> {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg:.green.bold} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
                .progress_chars("#>-"),
        );
        progress_bar.set_length(total);
        progress_bar.reset_elapsed();
        Ok(Self {
            inner: reader,
            progress_bar,
            uploaded: 0,
        })
    }
}

impl<'a, R: Read> Read for UploadTracker<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let bytes_read = self.inner.read(buf)?;
        self.uploaded += bytes_read;
        self.progress_bar.set_position(self.uploaded as u64);
        Ok(bytes_read)
    }
}

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

    /// Uploads the given URL (stream) to the server.
    pub fn upload_url(&self, url: &'a str) -> UploadResult<'a, String> {
        let field = if self.config.paste.oneshot == Some(true) {
            "oneshot_url"
        } else {
            "url"
        };

        if let Err(e) = Url::parse(url) {
            UploadResult(url, Err(e.into()))
        } else {
            let mut multipart = Multipart::new();
            multipart.add_stream::<_, &[u8], &str>(field, url.as_bytes(), None, None);
            UploadResult(url, self.upload(multipart))
        }
    }

    /// Uploads the given remote URL (stream) to the server.
    pub fn upload_remote_url(&self, url: &'a str) -> UploadResult<'a, String> {
        if let Err(e) = Url::parse(url) {
            UploadResult(url, Err(e.into()))
        } else {
            let mut multipart = Multipart::new();
            multipart.add_stream::<_, &[u8], &str>("remote", url.as_bytes(), None, None);
            UploadResult(url, self.upload(multipart))
        }
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
        if let Some(content_len) = multipart_data.content_len() {
            request = request.set("Content-Length", &content_len.to_string());
        }
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.set("Authorization", auth_token);
        }
        if let Some(expiration_time) = &self.config.paste.expire {
            request = request.set(EXPIRATION_HEADER, expiration_time);
        }
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(Duration::from_millis(80));
        progress_bar.set_message("Uploading");
        let upload_tracker = UploadTracker::new(
            &progress_bar,
            multipart_data.content_len().unwrap_or_default(),
            multipart_data,
        )?;
        let result = match request.send(upload_tracker) {
            Ok(response) => {
                let status = response.status();
                let response_text = response.into_string()?;
                if response_text.lines().count() != 1 {
                    Err(Error::UploadError(format!(
                        "server returned invalid body (status code: {status})"
                    )))
                } else if status == 200 {
                    Ok(response_text)
                } else {
                    Err(Error::UploadError(format!(
                        "unknown error (status code: {status})"
                    )))
                }
            }
            Err(UreqError::Status(code, response)) => Err(Error::UploadError(format!(
                "{} (status code: {})",
                response.into_string()?,
                code
            ))),
            Err(e) => Err(Error::RequestError(Box::new(e))),
        };
        progress_bar.finish_and_clear();
        result
    }

    /// Returns the server version.
    pub fn retrieve_version(&self) -> Result<String> {
        let mut url = Url::parse(&self.config.server.address)?;
        url.set_path("version");
        let mut request = self.client.get(url.as_str());
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.set("Authorization", auth_token);
        }
        Ok(request
            .call()
            .map_err(|e| Error::RequestError(Box::new(e)))?
            .into_string()?)
    }
}
