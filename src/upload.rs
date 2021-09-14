use crate::config::ServerConfig;
use crate::error::{Error, Result};
use multipart::client::lazy::Multipart;

/// Upload the given file to the server.
pub fn upload_file(file: &str, config: &ServerConfig) -> Result<String> {
    let mut multipart = Multipart::new();
    multipart.add_file("file", file);

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
