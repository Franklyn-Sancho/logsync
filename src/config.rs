use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LogMonitorConfig {
    pub log_file_path: PathBuf,
    pub monitored_file: PathBuf,
    pub high_priority_keywords: Vec<String>,
    pub very_high_priority_keywords: Vec<String>,
    pub check_interval_ms: u64,
}

impl Default for LogMonitorConfig {
    fn default() -> Self {
        Self {
            log_file_path: PathBuf::from("./filtered_log.json"),
            monitored_file: PathBuf::from("./test_log.txt"),
            high_priority_keywords: vec!["ERROR".to_string()],
            very_high_priority_keywords: vec!["CRITICAL".to_string()],
            check_interval_ms: 100,
        }
    }
}

impl LogMonitorConfig {
    pub fn new() -> Self {
        Default::default()
    }

    // Métodos para personalizar configuração
    pub fn with_log_file(mut self, path: PathBuf) -> Self {
        self.log_file_path = path;
        self
    }

    pub fn with_monitored_file(mut self, path: PathBuf) -> Self {
        self.monitored_file = path;
        self
    }
}