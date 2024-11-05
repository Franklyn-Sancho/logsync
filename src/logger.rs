use std::fs::{self, File as StdFile, OpenOptions};
use std::io::{self, BufRead, BufReader, Cursor, Read};
use std::path::Path;
use google_drive3::DriveHub;
use tokio::time::{self, Duration};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use google_drive3::api::File;
use std::io::Write;


use inotify::{Inotify, WatchMask};

pub async fn upload_file(hub: &DriveHub<HttpsConnector<HttpConnector>>, file_path: &str) -> Result<File, Box<dyn std::error::Error>> {
    // Assegura que o arquivo de log existe antes de tentar carregá-lo
    ensure_log_file_exists(file_path)?;

    // Abre o arquivo local
    let mut file = StdFile::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?; // Lê o arquivo inteiro no buffer

    // Cria um Cursor para permitir a leitura do buffer
    let cursor = Cursor::new(buffer);

    // Determina o tipo MIME do arquivo
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();

    // Prepara os metadados do arquivo para o Google Drive
    let drive_file = File {
        name: Some(Path::new(file_path).file_name().unwrap().to_string_lossy().to_string()), // Apenas o nome do arquivo
        mime_type: Some(mime_type.to_string()),
        ..Default::default()
    };

    // Faz o upload do arquivo usando o Cursor e o tipo MIME
    let (_, uploaded_file) = hub.files().create(drive_file)
        .upload(cursor, mime_type)
        .await?;

    Ok(uploaded_file) // Retorna o arquivo carregado
}

pub async fn monitor_and_upload(log_file_path: &str, hub: DriveHub<HttpsConnector<HttpConnector>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut inotify = Inotify::init()?;
    inotify.add_watch("/var/log/syslog", WatchMask::MODIFY)?;

    // Garantindo que `filtered_logs.txt` existe ou criando-o
    let mut output_file = OpenOptions::new().append(true).create(true).open(log_file_path)?;

    loop {
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(inotify::EventMask::MODIFY) {
                // Abrindo o log do sistema para leitura
                let file = StdFile::open("/var/log/syslog")?;
                let reader = BufReader::new(file);

                // Filtrando as linhas que contêm "ERROR" ou "WARN"
                let mut has_relevant_logs = false;
                for line in reader.lines() {
                    let line = line?;
                    if line.contains("ERROR") || line.contains("WARN") {
                        writeln!(output_file, "{}", line)?;
                        has_relevant_logs = true;
                    }
                }

                // Se encontrou logs relevantes, faz o upload
                if has_relevant_logs {
                    output_file.flush()?;
                    match upload_file(&hub, log_file_path).await {
                        Ok(uploaded_file) => println!("Uploaded file: {:?}", uploaded_file),
                        Err(e) => eprintln!("Error uploading file: {}", e),
                    }
                }
            }
        }

        // Pausa curta para evitar uso excessivo de CPU
        time::sleep(Duration::from_millis(100)).await;
    }
}

pub fn ensure_log_file_exists(file_path: &str) -> io::Result<()> {
    if !Path::new(file_path).exists() {
        // Cria o arquivo vazio se ele não existir
        let mut file = StdFile::create(file_path)?;
        writeln!(file, "")?; // Escreve uma linha vazia para criar o arquivo
    }
    Ok(())
}