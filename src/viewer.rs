use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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

use crate::{logger, utils};

#[derive(Clone, Debug)]
/// Represents a log entry with additional information.
pub struct LogEntry {
    pub timestamp: u64,
    pub log_type: String,
    pub priority: String,
    pub message: String,
    pub telegram_notification: Option<bool>, // Status of the Telegram alert notification
}

/// Starts the interactive viewer and handles terminal setup.
pub async fn start_interactive_viewer(mut rx: Receiver<LogEntry>, max_logs: usize) {
    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    if let Err(err) = run_app(&mut terminal, &mut rx, max_logs).await {
        eprintln!("Error: {:?}", err);
    }

    // Restore terminal settings after exiting the app
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}

/// Runs the application, processing logs and handling user input.
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    rx: &mut Receiver<LogEntry>,
    max_logs: usize, // Accepts the max number of logs as a parameter
) -> std::io::Result<()> {
    let mut logs = VecDeque::with_capacity(max_logs); // Stores logs with a configurable capacity
    let mut selected_log = None;
    let mut debug_messages = Vec::new();

    loop {
        // Process incoming logs
        while let Ok(log) = rx.try_recv() {
            if log.priority == "high" {
                // Ensure we don't exceed the maximum log capacity
                if logs.len() == max_logs {
                    logs.pop_front(); // Remove the oldest log if capacity is reached
                }
                logs.push_back(log.clone()); // Add the new log to the deque
                debug_messages.push(format!(
                    "Log added: {} - {}",
                    log.timestamp, log.message
                ));

                // Limit debug messages to avoid unbounded growth
                if debug_messages.len() > 10 {
                    debug_messages.remove(0);
                }
            }
        }

        // Draw the interface
        terminal.draw(|f| ui(f, &logs.as_slices().0, selected_log, &debug_messages))?;

        // Handle key events for navigation and quitting
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()), // Quit on 'q'
                    KeyCode::Down => {
                        // Navigate down in the log list, ensuring bounds
                        selected_log = selected_log.map_or(Some(0), |idx| {
                            Some((idx + 1).min(logs.len() - 1))
                        });
                    }
                    KeyCode::Up => {
                        // Navigate up, preventing negative indices
                        selected_log = selected_log.map_or(Some(0), |idx| {
                            Some(idx.saturating_sub(1))
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Renders the UI, including the log table, details, and debug messages.
fn ui<B: Backend>(
    f: &mut Frame<B>,
    logs: &[LogEntry],
    selected_log: Option<usize>,
    debug_messages: &[String],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(60), // Log table occupies 60% of the height
                Constraint::Percentage(20), // Log details take 20%
                Constraint::Percentage(20), // Debug messages take 20%
            ]
            .as_ref(),
        )
        .split(f.size());

        let rows = logs.iter().enumerate().map(|(i, log)| {
            let cells = vec![
                Cell::from(utils::format_timestamp(log.timestamp)),  // Usando a função format_timestamp
                Cell::from(log.log_type.clone()).style(get_color(&log.priority)),
                Cell::from(log.priority.clone()).style(get_color(&log.priority)),
                Cell::from(log.message.clone()),
                Cell::from(match log.telegram_notification {
                    Some(true) => "Alerta Enviado",
                    _ => "Não Enviado",
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

    f.render_widget(table, chunks[0]);

    // Display selected log details
    if let Some(idx) = selected_log {
        if idx < logs.len() {
            let log = &logs[idx];
            let mut details = vec![
                Spans::from(Span::raw(format!("Timestamp: {}", log.timestamp))),
                Spans::from(Span::raw(format!("Type: {}", log.log_type))),
                Spans::from(Span::raw(format!("Priority: {}", log.priority))),
                Spans::from(Span::raw(format!("Message: {}", log.message))),
            ];

            // Display Telegram notification status
            details.push(Spans::from(Span::styled(
                format!(
                    "Telegram Notification: {}",
                    match log.telegram_notification {
                        Some(true) => "Alert sent to Telegram",
                        _ => "Not sent",
                    }
                ),
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )));

            let details_paragraph = tui::widgets::Paragraph::new(details).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Log Details"),
            );
            f.render_widget(details_paragraph, chunks[1]);
        }
    }

    // Render debug messages
    let debug_paragraph = tui::widgets::Paragraph::new(
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
    f.render_widget(debug_paragraph, chunks[2]);
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