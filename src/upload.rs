use crate::config::ServerConfig;
use crate::error::{Error, Result};
use multipart::client::lazy::Multipart;
use std::io::Read;

/// Default file name to use for multipart stream.
const DEFAULT_FILE_NAME: Option<&str> = Some("file");

/// Uploads the given file to the server.
pub fn upload_file(file: &str, config: &ServerConfig) -> Result<String> {
    let mut multipart = Multipart::new();
    multipart.add_file("file", file);

    upload(multipart, config)
}

/// Uploads a stream to the server.
pub fn upload_stream<S: Read>(stream: S, config: &ServerConfig) -> Result<String> {
    let mut multipart = Multipart::new();
    multipart.add_stream("file", stream, DEFAULT_FILE_NAME, None);

    upload(multipart, config)
}

/// Uploads the given multipart data.
fn upload(mut multipart: Multipart<'static, '_>, config: &ServerConfig) -> Result<String> {
    let multipart_data = multipart.prepare()?;
    let mut request = ureq::post(&config.address).set(
        "Content-Type",
        &format!(
            "multipart/form-data; boundary={}",
            multipart_data.boundary()
        ),
    );
    if let Some(auth_token) = &config.auth_token {
        request = request.set("Authorization", auth_token);
    }
    let response = request
        .send(multipart_data)
        .map_err(|e| Error::RequestError(Box::new(e)))?;
    Ok(response.into_string()?)
}
