use google_drive3::api::File;
use google_drive3::DriveHub;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use inotify::{EventMask, Inotify, WatchMask};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::{File as StdFile, OpenOptions};
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

use tokio::time::Duration;

use crate::driver_uploader::upload_file;
use crate::notifier::{handle_telegram_alert, send_html_report_to_telegram, send_log_to_channel, send_telegram_alert};
use crate::parser::parse_log_line;
use crate::types::LogEntry;



async fn process_log_line(
    line: &str,
    log_file_path: &str,
    tx: &Sender<LogEntry>,
    processed_errors: &Arc<Mutex<HashSet<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let line_string = line.to_string();

    let mut should_process = false;
    {
        let mut guard = processed_errors.lock().unwrap();
        if !guard.contains(&line_string) {
            guard.insert(line_string.clone());
            should_process = true;
        }
    }

    if should_process {
        if let Some(log_json) = parse_log_line(line) {
            println!("Filtered log: {}", log_json);

            if log_json["priority"] == "high" {
                let log_entry = create_log_entry(&log_json)?;

                update_log_file(log_file_path, &log_entry)?;

                send_log_to_channel(tx, log_entry.clone()).await?;

                handle_telegram_alert(&log_entry).await?;
            }
        }
    }
    Ok(())
}

fn create_log_entry(log_json: &Value) -> Result<LogEntry, Box<dyn std::error::Error>> {
    Ok(LogEntry {
        timestamp: log_json["timestamp"].as_u64().ok_or("Invalid timestamp")?,
        log_type: log_json["type"].as_str().ok_or("Invalid type")?.to_string(),
        priority: log_json["priority"]
            .as_str()
            .ok_or("Invalid priority")?
            .to_string(),
        message: log_json["message"]
            .as_str()
            .ok_or("Invalid message")?
            .to_string(),
        telegram_notification: Some(true),
    })
}

fn update_log_file(
    log_file_path: &str,
    log_entry: &LogEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut logs: Vec<LogEntry> = if Path::new(log_file_path).exists() {
        let file = StdFile::open(log_file_path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
    } else {
        Vec::new()
    };

    logs.push(log_entry.clone());

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(log_file_path)?;

    serde_json::to_writer_pretty(&mut file, &logs)
        .map_err(|e| format!("Error writing to log file: {}", e))?;

    println!("Log entry written successfully.");
    Ok(())
}



pub async fn monitor_logs_and_create_json(
    log_file_path: &str,
    hub: &DriveHub<HttpsConnector<HttpConnector>>,
    tx: Sender<LogEntry>,
    processed_errors: Arc<Mutex<HashSet<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut inotify = Inotify::init()?;
    inotify.add_watch("./test_log.txt", WatchMask::MODIFY)?;

    println!("Monitoring ./test_log.txt for changes...");

    loop {
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                let file = StdFile::open("./test_log.txt")?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("Processing line: {}", line);

                        if let Err(e) =
                            process_log_line(&line, log_file_path, &tx, &processed_errors).await
                        {
                            eprintln!("Error processing log line: {}", e);
                        }
                    }
                }

                if let Err(e) = upload_file(hub, log_file_path).await {
                    eprintln!("Erro ao enviar arquivo: {}", e);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
