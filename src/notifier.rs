use reqwest;
use serde_json::json;
use std::env;

pub async fn send_telegram_alert(message: &str) -> Result<bool, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let token = env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN not set");
    let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID not set");

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .json(&json!({"chat_id": chat_id, "text": message}))
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                Ok(true)  // Retorna true se o envio foi bem-sucedido
            } else {
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
            eprintln!("Request error: {}", err);
            Err(Box::new(err))
        }
    }
}

