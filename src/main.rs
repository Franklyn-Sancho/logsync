use auth::authenticate;
use logger::{ensure_log_file_exists, monitor_and_upload};

mod auth;
mod logger;

#[tokio::main]
async fn main() {
    let drive_hub = match authenticate().await {
        Ok(hub) => hub,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            return;
        }
    };

    let log_file_path = "filtered_logs.txt"; // Path to your filtered logs file

    // Garante que o arquivo de log existe
    if let Err(e) = ensure_log_file_exists(&log_file_path) {
        eprintln!("Erro ao criar arquivo de log: {}", e);
        return;
    }

    if let Err(e) = monitor_and_upload(log_file_path, drive_hub).await {
        eprintln!("Error during monitoring and upload: {}", e);
    }
}
