# LOGSYNC - LOG SYNC APPLICATION IN RUST 

LogSync is a Rust application that monitors system log files in real-time, filtering and recording only error and warning messages. Designed to help users quickly identify and resolve issues, it enables easy access to relevant logs, even on mobile devices. Future features will include automatic integration with Google Drive for log backup.

## Features

- **Real-time system log monitoring**.
- **Automatic filtering** of messages containing `ERROR` or `WARN`.
- **Cloud backup** of the filtered logs, uploaded to Google Drive.

## Requirements

- Rust
- Cargo
- Google Drive API credentials
- Rust dependencies:
  - `yup-oauth2`
  - `google-drive3`
  - `hyper`
  - `tokio`
  - `inotify`

## Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Franklyn-Sancho/logsync.git
   cd LogSync
   ```
   

2. **Install dependencies:**
   ```bash
   cargo build
   ```
   


## Configuration & Usage

### Configuration

Ensure the `client_secret.json` file is placed in the root folder for Google Drive authentication. Also, ensure the path to the log file (e.g., `/var/log/syslog`) is accessible, or modify the code accordingly if a different log file path is needed.

### Running the Application

Run the application with the following command:

```bash
cargo run --release
```

The application will:

- Authenticate with Google Drive using the client_secret.json file.
- Monitor the log file (/var/log/syslog by default) for error and warning messages.
- Store the error and warning messages in filtered_logs.txt.
- Upload filtered_logs.txt to Google Drive whenever new errors or warnings are added.
  
## File Structure
1. **main.rs**: *The entry point of the application. It sets up the monitoring and authentication process*.
2. **logger.rs**: *Functions that monitor the system logs and write error/warning messages to filtered_logs.txt*.
3. **drive_integration.rs**: *Handles Google Drive authentication and uploading the logs*.
   
## Example of filtered_logs.txt
Once errors or warnings are detected in the logs, the filtered_logs.txt will look something like this:

```plaintext
Nov 5 12:34:56 myhost app[1234]: ERROR - Unable to connect to database
Nov 5 12:35:00 myhost app[1235]: WARN - Low memory warning
```

## Contributing & License

### Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

### License

This project is licensed under the MIT License. See the LICENSE file for more details.







