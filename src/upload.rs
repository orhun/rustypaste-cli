use crate::config::Config;
use crate::error::{Error, Result};
use indicatif::{ProgressBar, ProgressStyle};
use multipart::client::lazy::Multipart;
use serde::Deserialize;
use std::io::{Read, Result as IoResult, Write};
use std::time::Duration;
#[cfg(feature = "use-native-certs")]
use ureq::tls::{Certificate, RootCerts, TlsConfig};
use ureq::{Agent, SendBody};
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
    pub file_size: Option<u64>,
    /// Type of the item.
    #[serde(default = "item_type_default")]
    pub item_type: String,
    /// ISO8601 formatted date-time string of the creation timestamp.
    pub creation_date_utc: Option<String>,
    /// ISO8601 formatted date-time string of the expiration timestamp if one exists for this file.
    pub expires_at_utc: Option<String>,
}

// Needed for backwards compat with older servers that do not provide the item_type yet
fn item_type_default() -> String {
    "file".to_string()
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

impl<R: Read> Read for UploadTracker<'_, R> {
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
        let client_config = Agent::config_builder().user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
        #[cfg(feature = "use-native-certs")]
        let client_config =
            client_config.tls_config(TlsConfig::builder().root_certs(native_root_certs()).build());
        Self {
            client: client_config.build().into(),
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
        let mut request = self
            .client
            .post(&self.config.server.address)
            .config()
            .http_status_as_error(false)
            .build()
            .header(
                "Content-Type",
                format!(
                    "multipart/form-data; boundary={}",
                    multipart_data.boundary()
                ),
            );
        if let Some(content_len) = multipart_data.content_len() {
            request = request.header("Content-Length", content_len.to_string());
        }
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.header("Authorization", auth_token);
        }
        if let Some(expiration_time) = &self.config.paste.expire {
            request = request.header(EXPIRATION_HEADER, expiration_time);
        }
        if let Some(filename) = &self.config.paste.filename {
            request = request.header(FILENAME_HEADER, filename);
        }
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(Duration::from_millis(80));
        progress_bar.set_message("Uploading");
        let mut upload_tracker = UploadTracker::new(
            &progress_bar,
            multipart_data.content_len().unwrap_or_default(),
            multipart_data,
        )?;
        let result = match request.send(SendBody::from_reader(&mut upload_tracker)) {
            Ok(response) => {
                let status = response.status();
                let response_text = response.into_body().read_to_string()?;
                if status.is_client_error() || status.is_server_error() {
                    Err(Error::UploadError(format!(
                        "{} (status code: {})",
                        response_text.trim(),
                        status.as_u16()
                    )))
                } else if response_text.lines().count() != 1 {
                    Err(Error::UploadError(format!(
                        "server returned invalid body (status code: {status})"
                    )))
                } else if status.as_u16() == 200 {
                    Ok(response_text)
                } else {
                    Err(Error::UploadError(format!(
                        "unknown error (status code: {status})"
                    )))
                }
            }
            Err(e) => Err(Error::RequestError(e)),
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
        let mut request = self
            .client
            .delete(url.as_str())
            .config()
            .http_status_as_error(false)
            .build();
        if let Some(delete_token) = &self.config.server.delete_token {
            request = request.header("Authorization", delete_token);
        }
        let result = match request.call() {
            Ok(response) => {
                let status = response.status();
                let response_text = response.into_body().read_to_string()?;
                if status.as_u16() == 200 {
                    Ok(response_text)
                } else if status.as_u16() == 404 {
                    Err(Error::DeleteError(response_text.trim().to_string()))
                } else if status.is_client_error() || status.is_server_error() {
                    Err(Error::DeleteError(format!(
                        "{} (status code: {})",
                        response_text.trim(),
                        status.as_u16()
                    )))
                } else {
                    Err(Error::DeleteError(format!(
                        "unknown error (status code: {status})"
                    )))
                }
            }
            Err(e) => Err(Error::RequestError(e)),
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
            request = request.header("Authorization", auth_token);
        }
        Ok(request.call()?.body_mut().read_to_string()?)
    }

    /// Retrieves and prints the files on server.
    pub fn retrieve_list<Output: Write>(&self, output: &mut Output, prettify: bool) -> Result<()> {
        let url = self.retrieve_url("list")?;
        let mut request = self.client.get(url.as_str());
        if let Some(auth_token) = &self.config.server.auth_token {
            request = request.header("Authorization", auth_token);
        }
        let mut response = request.call()?;
        if !prettify {
            writeln!(output, "{}", response.body_mut().read_to_string()?)?;
            return Ok(());
        }
        let items: Vec<ListItem> = response.body_mut().read_json()?;
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
            .map(|v| v.file_size.unwrap_or_default())
            .max()
            .unwrap_or_default()
            .to_string()
            .len();
        if filesize_width < 4 {
            filesize_width = 4;
        }
        let mut itemtype_width = items
            .iter()
            .map(|v| v.item_type.len())
            .max()
            .unwrap_or_default();
        if itemtype_width < 4 {
            itemtype_width = 4;
        }
        writeln!(
            output,
            "{:^filename_width$} | {:^filesize_width$} | {:^itemtype_width$} | {:^19} | {:^19}",
            "Name", "Size", "Type", "Creation (UTC)", "Expiry (UTC)"
        )?;
        writeln!(
            output,
            "{:-<filename_width$}-|-{:->filesize_width$}-|-{:->itemtype_width$}-|-{:-<19}-|-{:-<19}",
            "", "", "", "", ""
        )?;
        items.iter().try_for_each(|file_info| {
            writeln!(
                output,
                "{:<filename_width$} | {:>filesize_width$} | {:<itemtype_width$} | {:<19} | {}",
                file_info.file_name,
                match file_info.file_size {
                    Some(size) => size.to_string(),
                    None => "n/a".to_string(),
                },
                file_info.item_type,
                file_info
                    .creation_date_utc
                    .as_deref()
                    .unwrap_or("info not available"),
                file_info.expires_at_utc.as_deref().unwrap_or_default()
            )
        })?;
        Ok(())
    }
}

#[cfg(feature = "use-native-certs")]
fn native_root_certs() -> RootCerts {
    let certs = rustls_native_certs::load_native_certs()
        .certs
        .iter()
        .map(|cert| Certificate::from_der(cert.as_ref()).to_owned())
        .collect::<Vec<_>>();
    RootCerts::new_with_certs(&certs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    fn test_server(status: &str, body: &str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let address = format!(
            "http://{}",
            listener
                .local_addr()
                .expect("test server should have a local address")
        );
        let status = status.to_string();
        let body = body.to_string();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("test server should accept a request");
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            let (header_len, content_len) = loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("test server should read request headers");
                request.extend_from_slice(&buffer[..bytes_read]);
                if let Some(header_end) = request.windows(4).position(|v| v == b"\r\n\r\n") {
                    let header_len = header_end + 4;
                    let headers = String::from_utf8_lossy(&request[..header_len]);
                    let content_len: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.strip_prefix("Content-Length: ")
                                .or_else(|| line.strip_prefix("content-length: "))
                        })
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_default();
                    break (header_len, content_len);
                }
            };
            while request.len() < header_len + content_len {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("test server should read the request body");
                request.extend_from_slice(&buffer[..bytes_read]);
            }
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .expect("test server should write the response");
        });
        (address, handle)
    }

    fn config(address: String) -> Config {
        let mut config = Config::default();
        config.server.address = address;
        config
    }

    #[test]
    fn upload_error_includes_response_body_and_status() {
        let (address, server) = test_server("422 Unprocessable Content", "invalid upload\n");
        let config = config(address);
        let result = Uploader::new(&config).upload_stream("content".as_bytes()).1;

        assert!(matches!(
            result,
            Err(Error::UploadError(message))
                if message == "invalid upload (status code: 422)"
        ));
        server.join().expect("test server should stop cleanly");
    }

    #[test]
    fn delete_not_found_uses_response_body() {
        let (address, server) = test_server("404 Not Found", "file does not exist\n");
        let config = config(address);
        let result = Uploader::new(&config).delete_file("missing").1;

        assert!(matches!(
            result,
            Err(Error::DeleteError(message)) if message == "file does not exist"
        ));
        server.join().expect("test server should stop cleanly");
    }

    #[test]
    fn version_request_keeps_status_errors_enabled() {
        let (address, server) = test_server("500 Internal Server Error", "failed\n");
        let config = config(address);
        let result = Uploader::new(&config).retrieve_version();

        assert!(matches!(result, Err(Error::RequestError(_))));
        server.join().expect("test server should stop cleanly");
    }
}
