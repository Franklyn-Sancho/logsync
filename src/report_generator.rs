use std::fs::File as StdFile;
use std::io::{BufWriter, Write};

use crate::types::LogEntry;

pub fn generate_html_report(log_entries: &Vec<LogEntry>) -> String {
    let mut html_content = String::new();
    html_content.push_str("<html><head><title>Error Report</title></head><body>");
    html_content.push_str("<h1>Error Report</h1>");
    html_content.push_str("<table border='1'><tr><th>Timestamp</th><th>Log Type</th><th>Priority</th><th>Message</th></tr>");

    for entry in log_entries {
        html_content.push_str("<tr>");
        html_content.push_str(&format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            entry.timestamp, entry.log_type, entry.priority, entry.message
        ));
        html_content.push_str("</tr>");
    }

    html_content.push_str("</table></body></html>");

    let file_name = "error_report.html";

    match StdFile::create(file_name) {
        Ok(file) => {
            let mut writer = BufWriter::new(file);
            if let Err(e) = writer.write_all(html_content.as_bytes()) {
                eprintln!("Error writing to HTML file: {}", e);
            }
        }
        Err(e) => eprintln!("Error creating HTML file: {}", e),
    }

    file_name.to_string()
}
