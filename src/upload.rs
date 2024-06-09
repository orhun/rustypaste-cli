use crate::config::Config;
use crate::error::{Error, Result};
use indicatif::{ProgressBar, ProgressStyle};
use multipart::client::lazy::Multipart;
use serde::Deserialize;
use std::io::{Read, Result as IoResult, Write};
use std::time::Duration;
use ureq::Error as UreqError;
use ureq::{Agent, AgentBuilder};
use url::Url;

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// HTTP header to use for specifying expiration times.
const EXPIRATION_HEADER: &str = "expire";

/// HTTP header for specifying the filename.
const FILENAME_HEADER: &str = "filename";

/// File entry item for list endpoint.
#[derive(Deserialize, Debug)]
pub struct ListItem {
    /// Uploaded file name.
    pub file_name: String,
    /// Size of the file in bytes.
    pub file_size: u64,
    /// ISO8601 formatted date-time string of the creation timestamp.
    #[serde(default = "creation_date_utc_default")]
    pub creation_date_utc: String,
    /// ISO8601 formatted date-time string of the expiration timestamp if one exists for this file.
    pub expires_at_utc: Option<String>,
}

fn creation_date_utc_default() -> String {
    "info not available".to_string()
}

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
        if let Some(filename) = &self.config.paste.filename {
            request = request.set(FILENAME_HEADER, filename);
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
                response.into_string()?.trim(),
                code
            ))),
            Err(e) => Err(Error::RequestError(Box::new(e))),
        };
        progress_bar.finish_and_clear();
        result
    }

    /// Wrapper: Delete the given file from the server.
    pub fn delete_file(&self, file: &'a str) -> UploadResult<'a, String> {
        UploadResult(file, self.delete(file))
    }

    /// Delete the given file from the server.
    fn delete(&self, file: &'a str) -> Result<String> {
        let url = self.retrieve_url(file)?;
        let mut request = self.client.delete(url.as_str());
        if let Some(delete_token) = &self.config.server.delete_token {
            request = request.set("Authorization", delete_token);
        }
        let result = match request.call() {
            Ok(response) => {
                let status = response.status();
                let response_text = response.into_string()?;
                if status == 200 {
                    Ok(response_text)
                } else {
                    Err(Error::DeleteError(format!(
                        "unknown error (status code: {status})"
                    )))
                }
            }
            Err(UreqError::Status(code, response)) => {
                if code == 404 {
                    Err(Error::DeleteError(
                        response.into_string()?.trim().to_string(),
                    ))
                } else {
                    Err(Error::DeleteError(format!(
                        "{} (status code: {})",
                        response.into_string()?.trim(),
                        code
                    )))
                }
            }
            Err(e) => Err(Error::RequestError(Box::new(e))),
        };
        result
    }

    /// Returns a valid request URL for an endpoint.
    pub fn retrieve_url(&self, endpoint: &str) -> Result<Url> {
        let mut url = Url::parse(&self.config.server.address)?;
        if !url.path().to_string().ends_with('/') {
            url = url.join(&format!("{}/", url.path()))?;
        }
        url = url.join(endpoint)?;
        Ok(url)
    }

    /// Returns the server version.
    pub fn retrieve_version(&self) -> Result<String> {
        let url = self.retrieve_url("version")?;
        let mut request = self.client.get(url.as_str());
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.set("Authorization", auth_token);
        }
        Ok(request
            .call()
            .map_err(|e| Error::RequestError(Box::new(e)))?
            .into_string()?)
    }

    /// Retrieves and prints the files on server.
    pub fn retrieve_list<Output: Write>(&self, output: &mut Output, prettify: bool) -> Result<()> {
        let url = self.retrieve_url("list")?;
        let mut request = self.client.get(url.as_str());
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.set("Authorization", auth_token);
        }
        let response = request
            .call()
            .map_err(|e| Error::RequestError(Box::new(e)))?;
        if !prettify {
            writeln!(output, "{}", response.into_string()?)?;
            return Ok(());
        }
        let items: Vec<ListItem> = response.into_json()?;
        if items.is_empty() {
            writeln!(output, "No files on server :(")?;
            return Ok(());
        }
        let filename_width = items
            .iter()
            .map(|v| v.file_name.len())
            .max()
            .unwrap_or_default();
        let mut filesize_width = items
            .iter()
            .map(|v| v.file_size)
            .max()
            .unwrap_or_default()
            .to_string()
            .len();
        if filesize_width < 4 {
            filesize_width = 4;
        }
        writeln!(
            output,
            "{:^filename_width$} | {:^filesize_width$} | {:^19} | {:^19}",
            "Name", "Size", "Creation (UTC)", "Expiry (UTC)"
        )?;
        writeln!(
            output,
            "{:-<filename_width$}-|-{:->filesize_width$}-|-{:-<19}-|-{:-<19}",
            "", "", "", ""
        )?;
        items.iter().try_for_each(|file_info| {
            writeln!(
                output,
                "{:<filename_width$} | {:>filesize_width$} | {:<19} | {}",
                file_info.file_name,
                file_info.file_size,
                file_info.creation_date_utc,
                file_info
                    .expires_at_utc
                    .as_ref()
                    .cloned()
                    .unwrap_or_default()
            )
        })?;
        Ok(())
    }
}
