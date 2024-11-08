
use google_drive3::api::File;
use google_drive3::DriveHub;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use inotify::{EventMask, Inotify, WatchMask};
use serde_json::json;
use std::fs::{File as StdFile, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{self, Duration};

use crate::notifier::send_telegram_alert;
use crate::utils::{ensure_file_exists, read_file_to_buffer};
use crate::viewer::LogEntry;

use tokio::sync::mpsc::Sender;

pub async fn monitor_logs_and_create_json(
    log_file_path: &str,
    hub: &DriveHub<HttpsConnector<HttpConnector>>,
    tx: Sender<LogEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut inotify = Inotify::init()?;
    inotify.add_watch("/var/log/syslog", WatchMask::MODIFY)?;

    let mut output_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file_path)?;

    loop {
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                let file = StdFile::open("/var/log/syslog")?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    if let Ok(line) = line {
                        if let Some(log_json) = log_to_json_and_alert(&line).await {
                            if log_json["priority"] == "high" {
                                writeln!(output_file, "{}", log_json.to_string())?;
                                output_file.flush()?;

                                let log_entry = LogEntry {
                                    timestamp: log_json["timestamp"].as_u64().unwrap(),
                                    log_type: log_json["type"].as_str().unwrap().to_string(),
                                    priority: log_json["priority"].as_str().unwrap().to_string(),
                                    message: log_json["message"].as_str().unwrap().to_string(),
                                    telegram_notification: Some(true),
                                };

                                // Envia o log para o canal
                                tx.send(log_entry.clone()).await.unwrap(); // Clonando antes de mover

                                // Agora, podemos usar o log_entry original
                                if let Some(true) = log_entry.telegram_notification {
                                    if let Err(err) = send_telegram_alert(&log_entry.message).await {
                                        eprintln!("Error sending alert to Telegram: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Err(e) = upload_file(hub, log_file_path).await {
                    eprintln!("Erro ao enviar arquivo: {}", e);
                }
            }
        }
        time::sleep(Duration::from_millis(100)).await;
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
