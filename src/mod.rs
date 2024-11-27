mod config;
mod models;
mod parser;
mod processor;
mod handlers;

pub use config::LogMonitorConfig;
pub use models::{LogEntry, LogType, LogPriority};
pub use processor::process_log_line;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};
use anyhow::{Context, Result};
use inotify::{Inotify, WatchMask, EventMask};
use tokio::sync::mpsc::Sender;
use tracing::{info, error};

pub async fn monitor_logs_and_create_json(
    hub: &DriveHub<HttpsConnector<HttpConnector>>,
    tx: Sender<LogEntry>,
    processed_errors: Arc<Mutex<HashSet<String>>>,
) -> Result<()> {
    let config = LogMonitorConfig::default();

    let mut inotify = Inotify::init()
        .context("Falha ao inicializar inotify")?;
    
    inotify.add_watch(&config.monitored_file, WatchMask::MODIFY)
        .context("Falha ao adicionar monitoramento de arquivo")?;

    info!("Monitorando {} para alterações...", config.monitored_file.display());

    loop {
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)
            .context("Erro ao ler eventos de arquivo")?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                let file = std::fs::File::open(&config.monitored_file)
                    .context("Falha ao abrir arquivo de log")?;
                let reader = std::io::BufReader::new(file);

                for line in reader.lines() {
                    if let Ok(line) = line {
                        info!("Processando linha: {}", line);

                        if let Err(e) = process_log_line(
                            &line, 
                            &config, 
                            &tx, 
                            &processed_errors
                        ).await {
                            error!("Erro ao processar linha de log: {}", e);
                        }
                    }
                }

                // Upload para Google Drive
                if let Err(e) = upload_file(hub, &config.log_file_path).await {
                    error!("Erro ao enviar arquivo: {}", e);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(config.check_interval_ms)).await;
    }
}