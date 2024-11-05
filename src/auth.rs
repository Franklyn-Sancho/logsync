use google_drive3::DriveHub;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Duration;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub async fn authenticate() -> Result<DriveHub<HttpsConnector<HttpConnector>>, Box<dyn std::error::Error>> {
    // Create an HTTPS connector
    let https = HttpsConnector::new();

    // Build the HTTP client with specified configurations
    let client = Client::builder(TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(30)) // Set idle timeout for the connection pool
        .http2_only(false) // Allow HTTP/1.1
        .build(https);

    // Load the configuration from client_secret.json
    let secret = yup_oauth2::read_application_secret("client_secret.json")
        .await
        .map_err(|_| "Error reading client_secret.json")?; // Handle errors in reading the secret file

    // Configure the authenticator
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json") // Save tokens to disk for future use
        .build()
        .await
        .expect("Error configuring the authenticator");

    // Define the required scopes for Google Drive access
    let scopes = &["https://www.googleapis.com/auth/drive.file"];

    // Attempt to obtain an access token for the specified scopes
    auth.token(scopes).await?; // Return the token directly, errors will propagate

    // Create the Google Drive hub with the HTTP client and authenticator
    let hub = DriveHub::new(client, auth);

    Ok(hub) // Return the created hub
}