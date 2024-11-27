use serde_json::Value;

use crate::{config::LogMonitorConfig, types::{LogEntry, LogPriority, LogType}};


pub fn parse_log_line(line: &str) -> Option<Value> {
    let priority = if line.contains("CRITICAL") {
        "very high"
    } else if line.contains("ERROR") {
        "high"
    } else {
        return None;
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let log_json = serde_json::json!({
        "timestamp": timestamp,
        "type": if line.contains("CRITICAL") { "CRITICAL" } else { "ERROR" },
        "priority": priority,
        "message": line.trim()
    });

    Some(log_json)
}

// Funções auxiliares de parsing podem ser adicionadas aqui
pub fn sanitize_log_message(message: &str) -> String {
    message.trim().to_string()
}