use std::fs::{File as StdFile};
use std::io::{self, Read, Write};
use std::path::Path;

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
