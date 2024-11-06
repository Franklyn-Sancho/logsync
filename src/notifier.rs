use reqwest;
use serde_json::json;
use std::env;

pub async fn send_test_message() -> Result<(), Box<dyn std::error::Error>> {

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Retrieve the Telegram API token and chat ID from environment variables
    let token = env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN not set");
    let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID not set");

    // Construct the Telegram API URL for sending the message
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    
    // Test message to be sent
    let message = "Test message from the Rust bot!";
    
    // Send the message using the HTTP client
    let client = reqwest::Client::new();
    client.post(&url)
        .json(&json!({"chat_id": chat_id, "text": message}))
        .send()
        .await?;

    // Confirm successful message sending
    println!("Test message sent successfully.");
    Ok(())
}

pub async fn send_telegram_alert(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Retrieve the Telegram API token and chat ID from environment variables
    let token = env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN not set");
    let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID not set");

    // Construct the URL for sending the message to Telegram's API
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

    // Create a new HTTP client
    let client = reqwest::Client::new();

    // Send the message via a POST request to Telegram API
    let response = client
        .post(&url)
        .json(&json!({"chat_id": chat_id, "text": message}))
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                // Confirm that the message was successfully sent
                println!("Alert message sent successfully!");
                Ok(())
            } else {
                // Log error if the response status is not successful
                eprintln!(
                    "Failed to send message to Telegram. Status: {}",
                    res.status()
                );
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to send Telegram alert",
                )))
            }
        }
        Err(err) => {
            // Log error if the request failed
            eprintln!("Request error: {}", err);
            Err(Box::new(err))
        }
    }
}
