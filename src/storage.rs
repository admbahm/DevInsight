use std::path::PathBuf;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{self, Write, BufReader, BufRead};
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct StoredLog {
    pub timestamp: DateTime<Local>,
    pub level: String,
    pub tag: String,
    pub message: String,
    pub device_id: Option<String>,
}

pub struct LogStorage {
    current_file: File,
    base_path: PathBuf,
    max_size: u64,
    current_size: u64,
}

impl LogStorage {
    pub fn new(base_path: PathBuf, max_size: u64) -> io::Result<Self> {
        create_dir_all(&base_path)?;
        let file_path = Self::generate_filename(&base_path);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        Ok(Self {
            current_file: file,
            base_path,
            max_size,
            current_size: 0,
        })
    }

    fn generate_filename(base_path: &PathBuf) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        base_path.join(format!("logcat_{}.jsonl", timestamp))
    }

    pub fn store_log(&mut self, log: StoredLog) -> io::Result<()> {
        let log_json = serde_json::to_string(&log)?;
        self.current_file.write_all(log_json.as_bytes())?;
        self.current_file.write_all(b"\n")?;
        
        self.current_size += log_json.len() as u64;
        if self.current_size >= self.max_size * 1024 * 1024 {
            self.rotate_log()?;
        }
        
        Ok(())
    }

    fn rotate_log(&mut self) -> io::Result<()> {
        let new_file_path = Self::generate_filename(&self.base_path);
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(new_file_path)?;
            
        self.current_file = new_file;
        self.current_size = 0;
        Ok(())
    }

    pub fn query_logs(&self, start_time: DateTime<Local>, end_time: DateTime<Local>) -> io::Result<Vec<StoredLog>> {
        let mut logs = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let reader = BufReader::new(File::open(entry.path())?);
            for line in reader.lines() {
                if let Ok(log_str) = line {
                    if let Ok(log) = serde_json::from_str::<StoredLog>(&log_str) {
                        if log.timestamp >= start_time && log.timestamp <= end_time {
                            logs.push(log);
                        }
                    }
                }
            }
        }
        Ok(logs)
    }
} 