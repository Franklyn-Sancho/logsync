use reqwest;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tokio::time;
use std::env;
use std::path::Path;
use std::time::Duration;



use reqwest::multipart;
use std::fs::File;
use std::io::Read;

use crate::types::LogEntry;

pub async fn send_log_to_channel(
    tx: &Sender<LogEntry>,
    log_entry: LogEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    tx.send(log_entry)
        .await
        .map_err(|e| format!("Error sending log to channel: {}", e).into())
}

pub async fn handle_telegram_alert(log_entry: &LogEntry) -> Result<(), Box<dyn std::error::Error>> {
    if log_entry.telegram_notification == Some(true) {
        send_telegram_alert(&log_entry.message)
            .await
            .map_err(|e| format!("Error sending alert to Telegram: {}", e).into())
    } else {
        Ok(())
    }
}

pub async fn send_html_report_to_telegram(report_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN not set");
    let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID not set");

    let url = format!("https://api.telegram.org/bot{}/sendDocument", token);
    let client = reqwest::Client::new();

    // Check if the HTML file exists
    if !Path::new(report_path).exists() {
        eprintln!("HTML report file does not exist at the given path: {}", report_path);
        return Err("File not found.".into());
    }

    // Open the HTML file
    let mut file = File::open(report_path)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    let report_path_string = report_path.to_string();

    let form = multipart::Form::new()
        .part("chat_id", multipart::Part::text(chat_id))
        .part("document", multipart::Part::bytes(file_content).file_name(report_path_string));

    let response = client
        .post(&url)
        .multipart(form)
        .timeout(Duration::from_secs(10)) // Add a 10-second timeout
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if status.is_success() {
        println!("Report sent successfully to Telegram.");
    } else {
        eprintln!("Failed to send report to Telegram. Status: {}. Body: {}", status, body);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to send report to Telegram")));
    }

    Ok(())
}

pub async fn send_telegram_alert(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let token = env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN not set");
    let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID not set");

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "text": message
        }))
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if status.is_success() {
        println!("Alert sent successfully to Telegram.");
        
        // Automatically send the report (without asking the user)
        let report_path = "error_report.html"; // Use the correct path to your HTML report
        if let Err(err) = send_html_report_to_telegram(report_path).await {
            eprintln!("Error sending HTML report to Telegram: {}", err);
            return Err(err);
        }

        Ok(())
    } else {
        eprintln!("Failed to send message to Telegram. Status: {}. Body: {}", status, body);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to send Telegram alert")))
    }
}




