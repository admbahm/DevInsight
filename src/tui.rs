use std::io;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Direction, Constraint},
    style::{Color, Style},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

pub struct LogEntry {
    pub level: LogLevel,
    pub timestamp: String,
    pub tag: String,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
    Unknown,
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            LogLevel::Error => Color::Red,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Info => Color::Green,
            LogLevel::Debug => Color::Blue,
            LogLevel::Verbose => Color::White,
            LogLevel::Unknown => Color::Gray,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Verbose => "VERBOSE",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    logs: Vec<LogEntry>,
    scroll: u16,
    rx: std::sync::mpsc::Receiver<LogEntry>,
}

impl Tui {
    pub fn new(rx: std::sync::mpsc::Receiver<LogEntry>) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            logs: Vec::new(),
            scroll: 0,
            rx,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            if let Ok(log) = self.rx.try_recv() {
                self.logs.push(log);
            }

            self.draw()?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                        KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Logs
                    Constraint::Length(1),  // Status line
                    Constraint::Length(3),  // Footer
                ])
                .split(size);

            // Header
            let header = Paragraph::new("DevInsight Log Viewer")
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // Logs with scrolling
            let log_count = self.logs.len();
            let visible_logs: Vec<ListItem> = self.logs.iter()
                .skip(self.scroll as usize)
                .map(|log| {
                    ListItem::new(format!(
                        "{} [{}] {}: {}",
                        log.timestamp,
                        log.tag,
                        log.level.as_str(),
                        log.message
                    )).style(Style::default().fg(log.level.color()))
                })
                .collect();

            let logs = List::new(visible_logs)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));
            frame.render_widget(logs, chunks[1]);

            // Status line
            let status = Paragraph::new(format!(
                "Total logs: {} | Scroll position: {}",
                log_count,
                self.scroll
            )).style(Style::default().fg(Color::White));
            frame.render_widget(status, chunks[2]);

            // Footer
            let footer = Paragraph::new("Press 'q' to quit, ↑/↓ to scroll")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(footer, chunks[3]);
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
    }
} 