use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

use inotify::{Inotify, WatchMask};



pub fn monitor_logs(log_path: &str, output_path: &str) -> io::Result<()> {
    let mut inotify = Inotify::init()?;
    inotify.add_watch(log_path, WatchMask::MODIFY)?;

    let mut output = File::create(output_path)?;

    loop {
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(inotify::EventMask::MODIFY) {
                let file = File::open(log_path)?;
                let reader = BufReader::new(file);
                
                for line in reader.lines() {
                    let line = line?;
                    if line.contains("ERROR") || line.contains("WARN") {
                        writeln!(output, "{}", line)?;
                    }
                }
            }
        }
    }
}