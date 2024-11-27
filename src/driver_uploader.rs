use google_drive3::DriveHub;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;

use google_drive3::api::File;

use std::path::Path;

use crate::utils::{ensure_file_exists, read_file_to_buffer};

// Function that uploads the log file to Google Drive
pub async fn upload_file(
    hub: &DriveHub<HttpsConnector<HttpConnector>>,
    file_path: &str,
) -> Result<File, Box<dyn std::error::Error>> {
    // Ensure the file exists
    ensure_file_exists(file_path)?;

    // Read the file content into a buffer
    let buffer = read_file_to_buffer(file_path)?;
    let cursor = std::io::Cursor::new(buffer);
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();

    // Prepare metadata for the Google Drive file
    let drive_file = File {
        name: Some(
            Path::new(file_path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ),
        mime_type: Some(mime_type.to_string()),
        ..Default::default()
    };

    // Upload the file to Google Drive
    let (_, uploaded_file) = hub
        .files()
        .create(drive_file)
        .upload(cursor, mime_type)
        .await?;

    Ok(uploaded_file)
}