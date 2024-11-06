use std::env;
use dotenv::dotenv;

use auth::authenticate;
use logger::monitor_and_log_to_json;
use serde_json::json;
use utils::ensure_file_exists;

mod auth;
mod logger;
mod notifier;
mod utils;

use crate::notifier::send_test_message;

#[tokio::main]
async fn main() {
    // Attempt to authenticate and obtain the drive hub
    let drive_hub = match authenticate().await {
        Ok(hub) => hub,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            return;
        }
    };

    // Path to the filtered logs JSON file
    let log_file_path = "filtered_logs.json";

    // Ensure the log file exists
    if let Err(e) = ensure_file_exists(&log_file_path) {
        eprintln!("Error creating log file: {}", e);
        return;
    }

    // Send a test message to verify the Telegram bot functionality
    if let Err(e) = send_test_message().await {
        eprintln!("Error sending test message: {}", e);
    }

    // Start monitoring the log and upload filtered logs
    if let Err(e) = monitor_and_log_to_json(log_file_path, &drive_hub).await {
        eprintln!("Error during log monitoring and upload: {}", e);
    }
}

