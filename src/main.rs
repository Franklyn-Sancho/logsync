mod auth;
mod logger;
mod notifier;
mod utils;
mod viewer;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Define the maximum number of logs to be stored
    let max_logs = 1000; // This can be any value you'd like or passed dynamically

    // Inicializa a autenticação do Google Drive
    let drive_hub = match auth::authenticate().await {
        Ok(hub) => hub,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            return;
        }
    };

    // Caminho do arquivo JSON onde os logs serão salvos
    let log_file_path = "filtered_logs.json";

    // Verifica se o arquivo de log existe, caso contrário, cria-o
    if let Err(e) = utils::ensure_file_exists(&log_file_path) {
        eprintln!("Error creating log file: {}", e);
        return;
    }

    // Cria um canal para enviar logs filtrados para exibição
    let (tx, rx) = mpsc::channel(100);

    // Inicia a monitoria dos logs em segundo plano
    let _monitor_task = tokio::spawn({
        let drive_hub = drive_hub.clone(); // Clona o hub para uso dentro da task
        async move {
            if let Err(e) =
                logger::monitor_logs_and_create_json(log_file_path, &drive_hub, tx).await
            {
                eprintln!("Error during log monitoring and upload: {}", e);
            }
        }
    });

    // Exibe os logs em tempo real no terminal interativo, passando a capacidade máxima de logs
    viewer::start_interactive_viewer(rx, max_logs).await;
}

