use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use thiserror::Error;
use colored::*;
use clap::Parser;
use chrono::NaiveDateTime;

#[derive(Error, Debug)]
pub enum DevInsightError {
    #[error("ADB not found or not accessible")]
    AdbNotFound,
    #[error("Failed to capture logcat output: {0}")]
    LogcatCaptureFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid timestamp format: {0}")]
    TimestampError(String),
}

#[derive(Parser, Debug)]
#[command(name = "DevInsight")]
#[command(author = "Adam Deane")]
#[command(version = "0.1.0")]
#[command(about = "Real-time Android Log Analyzer")]
struct Cli {
    #[arg(short, long, help = "Filter logs by error level (E, W, D, etc.)")]
    filter: Option<String>,
    
    #[arg(short, long, help = "Filter logs by specific tag")]
    tag: Option<String>,
    
    #[arg(short = 'c', long, help = "Clear logs before starting")]
    clear: bool,
    
    #[arg(short = 'T', long, help = "Show logs from specific timestamp (format: 'YYYY-MM-DD HH:MM:SS')")]
    since: Option<String>,
    
    #[arg(short = 'b', long = "buffer", help = "Select buffer (main, system, crash)", value_parser = ["main", "system", "crash"], default_value = "main")]
    buffer: String,
    
    #[arg(short = 'v', long = "format", help = "Log format (brief, process, tag, thread, raw)", value_parser = ["brief", "process", "tag", "thread", "raw"], default_value = "brief")]
    format: String,
}

struct LogProcessor {
    filter_level: Option<String>,
    filter_tag: Option<String>,
}

impl LogProcessor {
    fn new(filter_level: Option<String>, filter_tag: Option<String>) -> Self {
        Self {
            filter_level,
            filter_tag,
        }
    }

    fn should_process_log(&self, log: &str) -> bool {
        if let Some(level) = &self.filter_level {
            let level_pattern = format!(" {}/", level); // Brief format
            let alt_pattern = format!("/{} ", level);   // Tag format
            if !log.contains(&level_pattern) && !log.contains(&alt_pattern) {
                return false;
            }
        }

        if let Some(tag) = &self.filter_tag {
            if !log.contains(tag) {
                return false;
            }
        }

        true
    }

    fn format_log(&self, log: &str) -> String {
        // Remove debug prints
        let formatted = if log.contains("E/") || log.contains(" E ") || log.contains("Error:") {
            format!("{}  {}", "🔴".red().bold(), log.bright_red().bold())
        } else if log.contains("W/") || log.contains(" W ") || log.contains("Warning:") {
            format!("{}  {}", "⚠️".yellow().bold(), log.bright_yellow().bold())
        } else if log.contains("I/") || log.contains(" I ") {
            format!("{}  {}", "ℹ️".green(), log.bright_green())
        } else if log.contains("D/") || log.contains(" D ") {
            format!("{}  {}", "🔧".blue(), log.bright_blue())
        } else if log.contains("V/") || log.contains(" V ") {
            format!("{}  {}", "📝".white(), log.bright_white())
        } else {
            format!("{}  {}", "❓".normal(), log)
        };

        // Keep color override
        colored::control::set_override(true);
        formatted
    }
}

fn main() -> Result<(), DevInsightError> {
    // Force color output
    colored::control::set_override(true);
    
    println!("{}", "DevInsight: Android Log Analyzer".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    let cli = Cli::parse();
    let processor = LogProcessor::new(cli.filter.clone(), cli.tag.clone());

    println!("{}", "Starting DevInsight: Real-time Android Log Analyzer...".cyan().bold());

    // Clear logs if requested
    if cli.clear {
        // Clear logs using separate command
        Command::new("adb")
            .args(["logcat", "-c"])
            .output()
            .map_err(|_| DevInsightError::AdbNotFound)?;
        println!("{}", "Logs cleared.".green().bold());
    }

    // Build the adb command for monitoring
    let mut adb_command = Command::new("adb");
    adb_command.arg("logcat");

    // Add buffer selection - capture all buffers by default
    adb_command.args(&["-b", "all"]);

    // Add format selection
    adb_command.arg("-v").arg(&cli.format);

    // Print the command we're running (for debugging)
    println!("{}", "Running command:".cyan().bold());
    println!("{:?}", adb_command);

    // First check if adb is available
    if Command::new("adb").arg("devices").output().is_err() {
        return Err(DevInsightError::AdbNotFound);
    }

    let process = adb_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())  // Capture stderr too
        .spawn()
        .map_err(|_| DevInsightError::AdbNotFound)?;

    let stdout = process.stdout
        .ok_or(DevInsightError::LogcatCaptureFailed("Failed to capture stdout".to_string()))?;
    let reader = BufReader::new(stdout);

    // Print command info
    println!("{}", "Log Settings:".yellow().bold());
    println!("Buffer: All buffers");  // Changed from cli.buffer since we're using all
    println!("Format: {}", cli.format.blue());
    if let Some(f) = &cli.filter {
        println!("Filter Level: {}", f.blue());
    }
    if let Some(t) = &cli.tag {
        println!("Tag Filter: {}", t.blue());
    }
    println!("{}", "=".repeat(50).yellow());

    // Add a startup message to verify logging is working
    Command::new("adb")
        .args(["shell", "log", "-p", "i", "-t", "DevInsight", "Log monitoring started"])
        .output()
        .ok();

    for line in reader.lines() {
        match line {
            Ok(log) => {
                if processor.should_process_log(&log) {
                    println!("{}", processor.format_log(&log));
                }
            }
            Err(e) => {
                println!("{}", format!("Error reading log: {}", e).red().bold());
                break;
            }
        }
    }

    Ok(())
}
