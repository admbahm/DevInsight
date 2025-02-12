use std::io;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    layout::{Layout, Direction, Constraint, Rect},
    style::{Color, Style, Modifier},
    Terminal, Frame,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::collections::VecDeque;
use crate::storage::StorageUpdate;

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

    pub fn as_str(&self) -> &'static str {
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

// Update View enum
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum View {
    Logs,
    Stats,
    Storage,
}

// Add application state
pub struct AppState {
    pub current_view: View,
    pub logs: VecDeque<LogEntry>,
    pub filtered_logs: Vec<usize>,  // Indices into logs
    pub scroll: usize,
    pub paused: bool,
    pub search_query: String,
    pub search_mode: bool,
    pub storage_info: Option<StorageInfo>,
    pub stats: LogStats,
}

pub struct StorageInfo {
    pub current_file: String,
    pub total_size: u64,
    pub file_count: usize,
}

pub struct LogStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub verbose_count: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_view: View::Logs,
            logs: VecDeque::with_capacity(10000),  // Limit memory usage
            filtered_logs: Vec::new(),
            scroll: 0,
            paused: false,
            search_query: String::new(),
            search_mode: false,
            storage_info: None,
            stats: LogStats {
                error_count: 0,
                warning_count: 0,
                info_count: 0,
                debug_count: 0,
                verbose_count: 0,
            },
        }
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        if !self.paused {
            // Update statistics
            match entry.level {
                LogLevel::Error => self.stats.error_count += 1,
                LogLevel::Warning => self.stats.warning_count += 1,
                LogLevel::Info => self.stats.info_count += 1,
                LogLevel::Debug => self.stats.debug_count += 1,
                LogLevel::Verbose => self.stats.verbose_count += 1,
                LogLevel::Unknown => (), // Do nothing for unknown levels
            }

            // Add log and maintain size limit
            if self.logs.len() >= 10000 {
                self.logs.pop_front();
            }
            self.logs.push_back(entry);
            self.update_filtered_logs();
        }
    }

    fn update_filtered_logs(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_logs = (0..self.logs.len()).collect();
        } else {
            self.filtered_logs = self.logs
                .iter()
                .enumerate()
                .filter(|(_, log)| {
                    let search_term = self.search_query.to_lowercase();
                    let message_match = log.message.to_lowercase().contains(&search_term);
                    let tag_match = log.tag.to_lowercase().contains(&search_term);
                    let level_match = log.level.as_str().to_lowercase().contains(&search_term);
                    
                    message_match || tag_match || level_match
                })
                .map(|(i, _)| i)
                .collect();
        }
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
    log_rx: std::sync::mpsc::Receiver<LogEntry>,
    storage_rx: std::sync::mpsc::Receiver<StorageUpdate>,
}

impl Tui {
    pub fn new(
        log_rx: std::sync::mpsc::Receiver<LogEntry>,
        storage_rx: std::sync::mpsc::Receiver<StorageUpdate>,
    ) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: AppState::new(),
            log_rx,
            storage_rx,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            // Check for new logs
            while let Ok(log) = self.log_rx.try_recv() {
                self.state.add_log(log);
            }

            // Check for storage updates
            while let Ok(update) = self.storage_rx.try_recv() {
                self.state.storage_info = Some(StorageInfo {
                    current_file: update.current_file,
                    total_size: update.total_size,
                    file_count: update.file_count,
                });
            }

            self.draw()?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.state.search_mode {
                        match key.code {
                            KeyCode::Esc => {
                                self.state.search_mode = false;
                                self.state.search_query.clear();
                                self.state.update_filtered_logs();
                            }
                            KeyCode::Enter => {
                                self.state.search_mode = false;
                            }
                            KeyCode::Char(c) => {
                                self.state.search_query.push(c);
                                self.state.update_filtered_logs();
                            }
                            KeyCode::Backspace => {
                                if !self.state.search_query.is_empty() {
                                    self.state.search_query.pop();
                                    self.state.update_filtered_logs();
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('1') => self.state.current_view = View::Logs,
                            KeyCode::Char('2') => self.state.current_view = View::Stats,
                            KeyCode::Char('3') => self.state.current_view = View::Storage,
                            KeyCode::Char('/') => self.state.search_mode = true,
                            KeyCode::Char(' ') => self.state.paused = !self.state.paused,
                            KeyCode::Up => self.state.scroll = self.state.scroll.saturating_sub(1),
                            KeyCode::Down => {
                                if self.state.scroll < self.state.filtered_logs.len().saturating_sub(1) {
                                    self.state.scroll += 1;
                                }
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                let max_scroll = self.state.filtered_logs.len().saturating_sub(1);
                                self.state.scroll = max_scroll;
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                self.state.scroll = 0;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let state = &self.state; // Take a reference to avoid multiple borrows
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ])
                .split(f.size());

            Self::draw_tabs(f, chunks[0], state.current_view);
            
            match state.current_view {
                View::Logs => Self::draw_logs(f, chunks[1], state),
                View::Stats => Self::draw_stats(f, chunks[1], state),
                View::Storage => Self::draw_storage(f, chunks[1], state),
            }

            Self::draw_status(f, chunks[2], state);
            Self::draw_help(f, chunks[3]);
        })?;
        Ok(())
    }

    fn draw_tabs(f: &mut Frame, area: Rect, current_view: View) {
        let titles = vec!["Logs", "Stats", "Storage"];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Views"))
            .select(match current_view {
                View::Logs => 0,
                View::Stats => 1,
                View::Storage => 2,
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, area);
    }

    fn draw_logs(f: &mut Frame, area: Rect, state: &AppState) {
        let visible_logs: Vec<ListItem> = state.filtered_logs
            .iter()
            .filter_map(|&index| state.logs.get(index))
            .skip(state.scroll)
            .take(area.height as usize)
            .map(|log| {
                let line = format!(
                    "{} [{}] {}: {}",
                    log.timestamp,
                    log.tag,
                    log.level.as_str(),
                    log.message
                );
                
                // Highlight the matching text if in search mode
                let style = if state.search_mode && !state.search_query.is_empty() {
                    Style::default()
                        .fg(log.level.color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(log.level.color())
                };
                
                ListItem::new(line).style(style)
            })
            .collect();

        let title = if state.search_mode {
            format!("Log Output (Searching: '{}', {} matches)", 
                state.search_query,
                state.filtered_logs.len()
            )
        } else {
            format!("Log Output ({} logs)", state.filtered_logs.len())
        };

        let logs = List::new(visible_logs)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().bg(Color::DarkGray));
        f.render_widget(logs, area);
    }

    fn draw_stats(f: &mut Frame, area: Rect, state: &AppState) {
        let stats = format!(
            "\nLog Statistics:\n\
            \n\
            🔴 Errors:   {}\n\
            ⚠️  Warnings: {}\n\
            ℹ️  Info:     {}\n\
            🔧 Debug:    {}\n\
            📝 Verbose:  {}\n\
            \n\
            Total Logs: {}\n\
            Memory Usage: {} entries",
            state.stats.error_count,
            state.stats.warning_count,
            state.stats.info_count,
            state.stats.debug_count,
            state.stats.verbose_count,
            state.logs.len(),
            state.logs.capacity(),
        );

        let stats_widget = Paragraph::new(stats)
            .block(Block::default().borders(Borders::ALL).title("Statistics"))
            .style(Style::default().fg(Color::White));
        f.render_widget(stats_widget, area);
    }

    fn draw_storage(f: &mut Frame, area: Rect, state: &AppState) {
        let storage_info = if let Some(info) = &state.storage_info {
            format!(
                "\nStorage Information:\n\
                \n\
                Current File: {}\n\
                Total Size: {} MB\n\
                File Count: {}\n",
                info.current_file,
                info.total_size / (1024 * 1024),
                info.file_count,
            )
        } else {
            "\nStorage not enabled\n\nUse --save to enable log storage".to_string()
        };

        let storage_widget = Paragraph::new(storage_info)
            .block(Block::default().borders(Borders::ALL).title("Storage Status"))
            .style(Style::default().fg(Color::White));
        f.render_widget(storage_widget, area);
    }

    fn draw_status(f: &mut Frame, area: Rect, state: &AppState) {
        let status = if state.search_mode {
            format!("Search: {} | Press Enter to confirm or Esc to cancel", state.search_query)
        } else {
            format!(
                "Logs: {} | Scroll: {} | {} | View: {:?}",
                state.logs.len(),
                state.scroll,
                if state.paused { "PAUSED" } else { "RUNNING" },
                state.current_view,
            )
        };

        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(Color::White));
        f.render_widget(status_widget, area);
    }

    fn draw_help(f: &mut Frame, area: Rect) {
        let help_text = "1-3: Switch Views | Space: Pause | /: Search | ↑/↓: Scroll | End/G: Latest | Home/g: First | q: Quit";
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, area);
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