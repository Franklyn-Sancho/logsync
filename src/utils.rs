use std::fs::{File as StdFile};
use std::io::{self, Read, Write};
use std::path::Path;

use chrono::{NaiveDateTime, Utc};

pub fn format_timestamp(timestamp: u64) -> String {
    // Converte o timestamp Unix para NaiveDateTime
    let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    // Formata a data para um formato legível
    naive.format("%Y-%m-%d %H:%M:%S").to_string()
}


pub fn ensure_file_exists(file_path: &str) -> io::Result<()> {
    if !Path::new(file_path).exists() {
        StdFile::create(file_path)?.write_all(b"")?;
    }
    Ok(())
}

pub fn read_file_to_buffer(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = StdFile::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
