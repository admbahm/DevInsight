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
    pub level_filters: Vec<LogLevel>,  // Enabled log levels
    pub tail_mode: bool,  // Add this field
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
            level_filters: vec![  // Start with all levels enabled
                LogLevel::Error,
                LogLevel::Warning,
                LogLevel::Info,
                LogLevel::Debug,
                LogLevel::Verbose,
            ],
            tail_mode: true,  // Start with tail mode enabled
        }
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        if !self.paused {
            // Batch process logs for better performance
            if self.logs.len() >= 10000 {
                // Remove oldest 1000 logs when we hit the limit
                for _ in 0..1000 {
                    self.logs.pop_front();
                }
            }
            
            // Update statistics
            match entry.level {
                LogLevel::Error => self.stats.error_count += 1,
                LogLevel::Warning => self.stats.warning_count += 1,
                LogLevel::Info => self.stats.info_count += 1,
                LogLevel::Debug => self.stats.debug_count += 1,
                LogLevel::Verbose => self.stats.verbose_count += 1,
                LogLevel::Unknown => (),
            }

            self.logs.push_back(entry);
            self.update_filtered_logs();
        }
    }

    pub fn toggle_level(&mut self, level: LogLevel) {
        if let Some(pos) = self.level_filters.iter().position(|&l| l == level) {
            self.level_filters.remove(pos);
        } else {
            self.level_filters.push(level);
        }
        self.update_filtered_logs();
    }

    fn update_filtered_logs(&mut self) {
        self.filtered_logs = self.logs
            .iter()
            .enumerate()
            .filter(|(_, log)| {
                let level_match = self.level_filters.contains(&log.level);
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    let search_term = self.search_query.to_lowercase();
                    log.message.to_lowercase().contains(&search_term) ||
                    log.tag.to_lowercase().contains(&search_term) ||
                    log.level.as_str().to_lowercase().contains(&search_term)
                };
                
                level_match && search_match
            })
            .map(|(i, _)| i)
            .collect();

        // Update scroll position if in tail mode
        if self.tail_mode {
            self.scroll = self.filtered_logs.len().saturating_sub(1);
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
        const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut spinner_idx = 0;
        let mut collected = 0;
        let mut no_logs_count = 0;
        const INITIAL_BATCH_SIZE: usize = 50;
        const MAX_WAIT_CYCLES: usize = 20;  // About 1 second max wait

        while collected < INITIAL_BATCH_SIZE && no_logs_count < MAX_WAIT_CYCLES {
            self.terminal.draw(|f| {
                let area = f.size();
                let loading_area = Rect::new(
                    area.width.saturating_sub(40) / 2,
                    area.height.saturating_sub(3) / 2,
                    40.min(area.width),
                    3.min(area.height)
                );

                let status = if collected == 0 {
                    format!("{} Waiting for logs...", SPINNERS[spinner_idx])
                } else {
                    format!("{} Collecting logs {}/{}", SPINNERS[spinner_idx], collected, INITIAL_BATCH_SIZE)
                };
                
                let loading = Paragraph::new(status)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded))
                    .style(Style::default().fg(Color::Cyan))
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(loading, loading_area);
            })?;

            spinner_idx = (spinner_idx + 1) % SPINNERS.len();
            
            if let Ok(log) = self.log_rx.try_recv() {
                self.state.add_log(log);
                collected += 1;
                no_logs_count = 0;
            } else {
                no_logs_count += 1;
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        // Force initial update and scroll position
        self.state.update_filtered_logs();
        self.state.scroll = self.state.filtered_logs.len().saturating_sub(1);

        // Main event loop
        loop {
            // Process any new logs
            while let Ok(log) = self.log_rx.try_recv() {
                self.state.add_log(log);
                if self.state.tail_mode {
                    self.state.scroll = self.state.filtered_logs.len().saturating_sub(1);
                }
            }

            // Process storage updates
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
                            KeyCode::Char('t') => self.state.tail_mode = !self.state.tail_mode,
                            KeyCode::Up => {
                                self.state.tail_mode = false;  // Disable tail mode when manually scrolling
                                self.state.scroll = self.state.scroll.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                if self.state.scroll < self.state.filtered_logs.len().saturating_sub(1) {
                                    self.state.scroll += 1;
                                    // Re-enable tail mode when scrolling to bottom
                                    if self.state.scroll >= self.state.filtered_logs.len().saturating_sub(1) {
                                        self.state.tail_mode = true;
                                    }
                                }
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                let max_scroll = self.state.filtered_logs.len().saturating_sub(1);
                                self.state.scroll = max_scroll;
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                self.state.scroll = 0;
                            }
                            KeyCode::Char('e') => self.state.toggle_level(LogLevel::Error),
                            KeyCode::Char('w') => self.state.toggle_level(LogLevel::Warning),
                            KeyCode::Char('i') => self.state.toggle_level(LogLevel::Info),
                            KeyCode::Char('d') => self.state.toggle_level(LogLevel::Debug),
                            KeyCode::Char('v') => self.state.toggle_level(LogLevel::Verbose),
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let state = &self.state;
        self.terminal.draw(|f| {
            // Get terminal size
            let size = f.size();
            
            // Create a main block for the entire UI
            let main_block = Block::default()
                .borders(Borders::NONE)
                .style(Style::default());
            
            // Create main layout with fixed margins
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Tabs
                    Constraint::Min(5),     // Main content (minimum 5 lines)
                    Constraint::Length(1),  // Status
                    Constraint::Length(3),  // Help
                ].as_ref())
                .horizontal_margin(1)       // Add horizontal margin
                .vertical_margin(0)         // No vertical margin
                .split(size);

            // Render each section within the main block
            f.render_widget(main_block, size);
            Self::draw_tabs(f, main_layout[0], state.current_view);
            
            // Create content area with proper borders
            let content_area = main_layout[1];
            let inner_area = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .inner(content_area);
            
            // Render appropriate content
            match state.current_view {
                View::Logs => Self::draw_logs(f, inner_area, state),
                View::Stats => Self::draw_stats(f, inner_area, state),
                View::Storage => Self::draw_storage(f, inner_area, state),
            }

            Self::draw_status(f, main_layout[2], state);
            Self::draw_help(f, main_layout[3]);
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
        // Calculate actual display area accounting for borders and padding
        let inner_width = area.width.saturating_sub(2);  // Subtract 2 for borders
        let max_display = area.height.saturating_sub(2); // Subtract 2 for borders
        let total_logs = state.filtered_logs.len();
        
        // Calculate the start index for displaying logs
        let start_index = if state.tail_mode {
            total_logs.saturating_sub(max_display as usize)
        } else {
            state.scroll
        };

        let visible_logs: Vec<ListItem> = state.filtered_logs
            .iter()
            .skip(start_index)
            .take(max_display as usize)
            .filter_map(|&index| state.logs.get(index))
            .map(|log| {
                // Fixed widths for each component
                const TIMESTAMP_WIDTH: usize = 19;
                const TAG_WIDTH: usize = 8;
                const LEVEL_WIDTH: usize = 5;
                const PADDING: usize = 7;  // For brackets, spaces, and colon

                // Calculate remaining width for message
                let message_width = (inner_width as usize)
                    .saturating_sub(TIMESTAMP_WIDTH)
                    .saturating_sub(TAG_WIDTH)
                    .saturating_sub(LEVEL_WIDTH)
                    .saturating_sub(PADDING)
                    .saturating_sub(2);  // Account for icon and space

                // Get the icon for the log level
                let icon = match log.level {
                    LogLevel::Error => "🔴",
                    LogLevel::Warning => "⚠️",
                    LogLevel::Info => "ℹ️",
                    LogLevel::Debug => "🔧",
                    LogLevel::Verbose => "📝",
                    LogLevel::Unknown => "❓",
                };

                let line = format!(
                    "{} {:<width$} [{:<tag_width$}] {:<level_width$}: {:.message_width$}",
                    icon,
                    log.timestamp,
                    log.tag.chars().take(TAG_WIDTH).collect::<String>(),
                    log.level.as_str(),
                    log.message,
                    width = TIMESTAMP_WIDTH,
                    tag_width = TAG_WIDTH,
                    level_width = LEVEL_WIDTH,
                    message_width = message_width
                );
                
                ListItem::new(line).style(Style::default().fg(log.level.color()))
            })
            .collect();

        let title = if state.search_mode {
            format!(" Log Output (Searching: '{}', {} matches) ", 
                state.search_query,
                state.filtered_logs.len()
            )
        } else {
            format!(" Log Output ({} logs) ", state.filtered_logs.len())
        };

        let logs = List::new(visible_logs)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_type(ratatui::widgets::BorderType::Rounded))
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
            let filters = format!("[{}{}{}{}{}]",
                if state.level_filters.contains(&LogLevel::Error) { "E" } else { "-" },
                if state.level_filters.contains(&LogLevel::Warning) { "W" } else { "-" },
                if state.level_filters.contains(&LogLevel::Info) { "I" } else { "-" },
                if state.level_filters.contains(&LogLevel::Debug) { "D" } else { "-" },
                if state.level_filters.contains(&LogLevel::Verbose) { "V" } else { "-" },
            );
            format!(
                "Logs: {} | Filters: {} | Scroll: {} | {} | {} | View: {:?}",
                state.logs.len(),
                filters,
                state.scroll,
                if state.paused { "PAUSED" } else { "RUNNING" },
                if state.tail_mode { "TAIL" } else { "SCROLL" },
                state.current_view,
            )
        };

        let status_widget = Paragraph::new(status)
            .style(Style::default().fg(Color::White));
        f.render_widget(status_widget, area);
    }

    fn draw_help(f: &mut Frame, area: Rect) {
        let help_text = "1-3: Views | Space: Pause | t: Tail | /: Search | e/w/i/d/v: Toggle Filters | ↑/↓: Scroll | End/G: Latest | Home/g: First | q: Quit";
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