use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use std::{collections::VecDeque, io, time::Duration};
use tokio::sync::mpsc;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

use tokio::sync::mpsc::Receiver;

use crate::utils;

#[derive(Clone, Debug)]
/// Represents a log entry with additional information.
pub struct LogEntry {
    pub timestamp: u64,
    pub log_type: String,
    pub priority: String,
    pub message: String,
    pub telegram_notification: Option<bool>, // Status of the Telegram alert notification
}

/// Starts an interactive viewer that displays logs in the terminal.
pub async fn start_interactive_viewer(mut rx: Receiver<LogEntry>, max_logs: usize) {
    enable_raw_mode().unwrap(); // Enable raw mode for terminal input handling
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap(); // Switch to alternate screen buffer
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap(); // Create terminal object

    loop {
        // Clear the screen on each render to ensure proper layout
        terminal.clear().unwrap();

        // Call run_app to display logs
        if let Err(e) = run_app(&mut terminal, &mut rx, max_logs).await {
            eprintln!("Error: {:?}", e);
        }
    }
}

/// Runs the application to display logs and handle user input.
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    rx: &mut mpsc::Receiver<LogEntry>,
    max_logs: usize,
) -> io::Result<()> {
    let mut logs = VecDeque::with_capacity(max_logs); // Using VecDeque to store logs
    let mut selected_log = Some(0);
    let mut debug_messages = Vec::new();
    let mut scroll_offset = 0;  // Variable to control scroll position

    loop {
        // Process received logs
        while let Ok(log) = rx.try_recv() {
            if log.priority == "high" {
                // Add log to the deque and remove the oldest if the limit is reached
                if logs.len() == max_logs {
                    logs.pop_front(); // Remove the oldest log
                }
                logs.push_back(log.clone()); // Add the new log
                debug_messages.push(format!(
                    "Log added: {} - {}",
                    log.timestamp, log.message
                ));

                // Limit the number of debug messages to avoid unbounded growth
                if debug_messages.len() > 10 {
                    debug_messages.remove(0);
                }
            }
        }

        // Draw the interface with logs, selected log, and debug messages
        terminal.draw(|f| ui(f, &logs.as_slices().0, selected_log, &debug_messages, scroll_offset))?;

        // Capture keyboard events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()), // Close the program with 'q'
                    KeyCode::Char('c') if key.modifiers == event::KeyModifiers::CONTROL => return Ok(()), // Close with Ctrl+C
                    KeyCode::Down => {
                        if logs.len() > 0 {
                            // Scroll down control
                            scroll_offset = (scroll_offset + 1).min(logs.len() - 1);
                        }
                    }
                    KeyCode::Up => {
                        if scroll_offset > 0 {
                            // Scroll up control
                            scroll_offset -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// UI rendering function to display logs and selected log details.
fn ui<B: Backend>(
    f: &mut Frame<B>,
    logs: &[LogEntry],
    selected_log: Option<usize>,
    debug_messages: &[String],
    scroll_offset: usize,
) {
    // Divide the screen into three sections: logs & details, debug messages, and instructions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(70), // 70% for logs and details
                Constraint::Min(5), // Minimum space for debug messages
                Constraint::Length(3), // Fixed space for instructions
            ]
            .as_ref(),
        )
        .split(f.size());

    // Split the main area (70%) into two parts: logs and log details
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(75), // 75% for logs
                Constraint::Percentage(25), // 25% for details
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    // Render the logs table in the first part
    let rows = logs.iter().skip(scroll_offset).enumerate().map(|(i, log)| {
        let cells = vec![
            Cell::from(utils::format_timestamp(log.timestamp)),
            Cell::from(log.log_type.clone()).style(get_color(&log.priority)),
            Cell::from(log.priority.clone()).style(get_color(&log.priority)),
            Cell::from(log.message.clone()),
            Cell::from(match log.telegram_notification {
                Some(true) => "Alert Sent",
                _ => "Not Sent",
            }),
        ];
        let row = Row::new(cells);
        if Some(i) == selected_log {
            row.style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            row
        }
    });

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                Cell::from("Timestamp"),
                Cell::from("Type"),
                Cell::from("Priority"),
                Cell::from("Message"),
                Cell::from("Telegram Notification"),
            ])
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Percentage(50),
            Constraint::Length(20),
        ]);

    f.render_widget(table, main_chunks[0]);

    // Display the selected log details next to the logs table
    if let Some(idx) = selected_log {
        if idx < logs.len() {
            let log = &logs[idx];
            let mut details = vec![
                Spans::from(Span::raw(format!("Timestamp: {}", log.timestamp))),
                Spans::from(Span::raw(format!("Type: {}", log.log_type))),
                Spans::from(Span::raw(format!("Priority: {}", log.priority))),
                Spans::from(Span::raw(format!("Message: {}", log.message))),
            ];

            details.push(Spans::from(Span::styled(
                format!(
                    "telegram_notification: {}",
                    match log.telegram_notification {
                        Some(true) => "Alert sent to Telegram",
                        _ => "Not sent",
                    }
                ),
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )));

            let details_paragraph = Paragraph::new(details).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Log Details"),
            );
            f.render_widget(details_paragraph, main_chunks[1]);
        }
    }

    // Render debug messages at the bottom of the screen
    let debug_paragraph = Paragraph::new(
        debug_messages
            .iter()
            .map(|msg| Spans::from(Span::raw(msg.clone())))
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Debug Messages"),
    );
    f.render_widget(debug_paragraph, chunks[1]);

    // Render navigation and quit instructions at the bottom of the screen
    let instructions = vec![
        Spans::from(Span::raw("Use Up/Down arrows to scroll logs")),
        Spans::from(Span::raw("Press Ctrl+C to quit")),
    ];
    let instructions_paragraph = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(instructions_paragraph, chunks[2]);
}

/// Returns a color style based on the log priority.
fn get_color(priority: &str) -> Style {
    match priority {
        "high" => Style::default().fg(Color::Red),
        "medium" => Style::default().fg(Color::Yellow),
        "low" => Style::default().fg(Color::Green),
        _ => Style::default(),
    }
}
