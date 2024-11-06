use dotenv::dotenv;
use google_drive3::api::File;
use google_drive3::DriveHub;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::fs::{self, File as StdFile, OpenOptions};
use std::io::Write;
use std::io::{self, BufRead, BufReader, Cursor, Read};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{self, Duration};

use crate::notifier::send_telegram_alert;
use crate::utils::{ensure_file_exists, read_file_to_buffer};


// Function that monitors system log for updates and filters relevant logs for upload
pub async fn monitor_and_log_to_json(
    log_file_path: &str,
    hub: &DriveHub<HttpsConnector<HttpConnector>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize file monitoring with inotify
    let mut inotify = inotify::Inotify::init()?;
    inotify.add_watch("/var/log/syslog", inotify::WatchMask::MODIFY)?;

    // Open log file for appending filtered logs
    let mut output_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file_path)?;

    loop {
        // Buffer to read inotify events
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        // Iterate over events
        for event in events {
            // If the file was modified, process it
            if event.mask.contains(inotify::EventMask::MODIFY) {
                let file = StdFile::open("/var/log/syslog")?;
                let reader = BufReader::new(file);

                // Filter logs and convert to JSON format
                let mut filtered_logs = Vec::new();
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if let Some(log_json) = log_to_json_and_alert(&line).await {
                            filtered_logs.push(log_json);
                        }
                    }
                }

                // If filtered logs exist, write them to the output file and upload
                if !filtered_logs.is_empty() {
                    for log in &filtered_logs {
                        writeln!(output_file, "{}", log.to_string())?;
                    }
                    output_file.flush()?;

                    // Upload the updated log file to Google Drive
                    if let Err(e) = upload_file(hub, log_file_path).await {
                        eprintln!("Error uploading file: {}", e);
                    }
                }
            }
        }

        // Sleep to avoid high CPU usage
        time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

// Function that converts the log line to JSON and sends alerts if necessary
pub async fn log_to_json_and_alert(line: &str) -> Option<serde_json::Value> {
    // Determine log type and priority based on the content
    let (log_type, priority) = if line.contains("CRITICAL") {
        ("CRITICAL", "high")
    } else if line.contains("ERROR") {
        ("ERROR", "medium")
    } else if line.contains("WARN") {
        ("WARNING", "low")
    } else {
        return None;
    };

    // Send a critical error alert if applicable
    if log_type == "CRITICAL" {
        let message = format!("Critical Error Detected: {}", line);
        match send_telegram_alert(&message).await {
            Ok(_) => println!("Critical error alert sent successfully!"),
            Err(e) => eprintln!("Error sending critical error alert: {}", e),
        }
    }

    // Return the log in JSON format
    Some(json!({
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        "type": log_type,
        "message": line.trim(),
        "priority": priority
    }))
}

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
