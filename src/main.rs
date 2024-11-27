mod auth;
mod logger;
mod notifier;
mod utils;
mod viewer;
mod driver_uploader;
mod types;
mod config;
mod parser;
mod processor;

use std::{collections::HashSet, sync::{Arc, Mutex}};

use logger::monitor_logs_and_create_json;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Define o número máximo de logs a serem armazenados
    let max_logs = 1000; // Este valor pode ser alterado conforme necessário

    // Inicializa a autenticação do Google Drive
    let drive_hub = auth::authenticate().await?;

    // Caminho do arquivo JSON onde os logs serão salvos
    let log_file_path = "filtered_logs.json";

    // Verifica se o arquivo de log existe, caso contrário, cria-o
    utils::ensure_file_exists(&log_file_path)?;

    // Cria um canal para enviar logs filtrados para exibição
    let (tx, rx) = mpsc::channel(100);

    // Cria um estado compartilhado para rastrear erros processados
    let processed_errors: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Inicia a monitoria dos logs em segundo plano
    let _monitor_task = {
        let drive_hub = drive_hub.clone();
        let processed_errors = Arc::clone(&processed_errors);
        let log_file_path = log_file_path.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = monitor_logs_and_create_json(&log_file_path, &drive_hub, tx, processed_errors).await {
                eprintln!("Error during log monitoring and upload: {}", e);
            }
        })
    };

    // Exibe os logs em tempo real no terminal interativo
    let _ = viewer::start_interactive_viewer(rx, max_logs).await;

    Ok(())
}

