use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{collections::VecDeque, io, time::Duration};
use tokio::sync::mpsc::{self, error::TryRecvError};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

use tokio::sync::mpsc::Receiver;

use crate::{types::LogEntry, utils};

/// Starts an interactive viewer that displays logs in the terminal.
pub async fn start_interactive_viewer(mut rx: Receiver<LogEntry>, max_logs: usize) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    println!("Viewer started, waiting for logs..."); // Debug log

    loop {
        // Clear the screen on each render to ensure proper layout
        terminal.clear()?;

        // Call run_app to display logs
        if let Err(e) = run_app(&mut terminal, &mut rx, max_logs).await {
            eprintln!("Error in viewer: {:?}", e);
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Runs the application to display logs and handle user input.
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    rx: &mut mpsc::Receiver<LogEntry>,
    max_logs: usize,
) -> io::Result<()> {
    let mut logs = VecDeque::with_capacity(max_logs);
    let mut debug_messages = Vec::new();
    let mut scroll_offset = 0;
    let mut selected_log = 0;

    loop {
        // Process received logs
        match rx.try_recv() {
            Ok(log) => {
                println!("Received log: {:?}", log); // Debug log
                if logs.len() == max_logs {
                    logs.pop_front();
                }
                logs.push_back(log);
                debug_messages.push(format!("New log received at {}", chrono::Local::now()));
                if debug_messages.len() > 10 {
                    debug_messages.remove(0);
                }
            }
            Err(TryRecvError::Empty) => {
                // Channel is empty, continue normally
            }
            Err(TryRecvError::Disconnected) => {
                debug_messages.push("Channel disconnected!".to_string());
            }
        }

        // Atualiza a posição máxima permitida para `selected_log`
        let max_index = logs.len().saturating_sub(1);
        selected_log = selected_log.min(max_index);

        // Renderiza a interface
        terminal.draw(|f| {
            ui(f, &logs.as_slices().0, Some(selected_log), &debug_messages, scroll_offset)
        })?;

        // Captura eventos do teclado
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()), // Sai com 'q'
                    KeyCode::Char('c') if key.modifiers == event::KeyModifiers::CONTROL => return Ok(()), // Sai com Ctrl+C
                    KeyCode::Down => {
                        if selected_log < max_index {
                            selected_log += 1; // Avança para o próximo log
                        }
                        // Ajusta a rolagem caso o cursor desça além da área visível
                        if selected_log >= scroll_offset + 10 {
                            scroll_offset += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected_log > 0 {
                            selected_log -= 1; // Retorna ao log anterior
                        }
                        // Ajusta a rolagem caso o cursor suba além da área visível
                        if selected_log < scroll_offset {
                            scroll_offset = scroll_offset.saturating_sub(1);
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
