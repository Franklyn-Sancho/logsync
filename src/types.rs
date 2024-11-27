use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub log_type: String,
    pub priority: String,
    pub message: String,
    pub telegram_notification: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogType {
    Error,
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum LogPriority {
    Low,
    Medium,
    High,
    VeryHigh,
}