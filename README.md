# DevInsight: Real-time Log Analyzer for Developers 🚀

![Build Status](https://img.shields.io/github/actions/workflow/status/YOUR_GITHUB/DevInsight/build.yml?branch=main)
![Contributors](https://img.shields.io/github/contributors/YOUR_GITHUB/DevInsight)
![License](https://img.shields.io/github/license/YOUR_GITHUB/DevInsight)

## Overview
**DevInsight** is a blazing-fast, Rust-powered log analysis tool designed for developers who need real-time Android log monitoring. Starting with **Android logcat**, DevInsight provides color-coded output, advanced filtering, and intelligent insights. Future expansions will include support for iOS syslogs, Docker logs, and cloud-based logging solutions.

## Features
✅ **Real-time log streaming** with color-coded output
✅ **Advanced filtering** by log level and tags
✅ **Multiple buffer support** (main, system, crash)
✅ **Flexible output formats**
✅ **Timestamp-based filtering**
✅ **Optimized Rust performance** for low-latency processing

## Installation

### Prerequisites
- Rust and Cargo
- Android Debug Bridge (ADB)
- Connected Android device or emulator

### Ubuntu/Linux
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install ADB for Android Debugging
sudo apt update && sudo apt install android-tools-adb -y

# Clone DevInsight Repository
git clone https://github.com/YOUR_GITHUB/DevInsight.git
cd DevInsight
cargo build --release
```

### macOS
```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
brew install rust

# Install ADB
brew install android-platform-tools

# Clone DevInsight Repository
git clone https://github.com/YOUR_GITHUB/DevInsight.git
cd DevInsight
cargo build --release
```

## Usage

### Basic Commands
```bash
# Basic log viewing
cargo run

# Force color output (if needed)
FORCE_COLOR=1 cargo run
```

### Filtering Options
```bash
# Filter by log level
cargo run -- --filter E    # Show only errors
cargo run -- --filter W    # Show only warnings
cargo run -- --filter I    # Show only info
cargo run -- --filter D    # Show only debug
cargo run -- --filter V    # Show only verbose

# Filter by tag
cargo run -- --tag MyApp   # Show logs only from 'MyApp'

# Combine filters
cargo run -- --filter E --tag MyApp  # Show only errors from 'MyApp'
```

### Buffer Selection
```bash
# Select specific buffer
cargo run -- --buffer main    # Main buffer only
cargo run -- --buffer system  # System buffer only
cargo run -- --buffer crash   # Crash buffer only
```

### Output Formatting
```bash
# Change log format
cargo run -- --format brief    # Brief format
cargo run -- --format process  # Show process ID
cargo run -- --format thread   # Show thread info
cargo run -- --format raw      # Raw log output
cargo run -- --format tag      # Tag-focused format
```

### Log Management
```bash
# Clear logs before starting
cargo run -- --clear

# Show logs since specific time
cargo run -- --since "2024-03-20 10:00:00"
```

### Color Coding
The output is color-coded for better readability:
- 🔴 **Red** - Errors (E)
- ⚠️ **Yellow** - Warnings (W)
- ℹ️ **Green** - Info (I)
- 🔧 **Blue** - Debug (D)
- 📝 **White** - Verbose (V)

## Command Line Options
| Option | Short | Description |
|--------|--------|-------------|
| `--filter` | `-f` | Filter logs by level (E, W, I, D, V) |
| `--tag` | `-t` | Filter logs by specific tag |
| `--clear` | `-c` | Clear logs before starting |
| `--since` | `-T` | Show logs since timestamp |
| `--buffer` | `-b` | Select buffer (main, system, crash) |
| `--format` | `-v` | Set output format |

## Roadmap
🚀 **Phase 1 (Android Logcat MVP)**
- ✅ Basic CLI for streaming logs
- ✅ Error and warning filtering
- ✅ Custom tag-based search
- ⏳ Interactive TUI interface using `ratatui`

⚡ **Phase 2 (Advanced Logging & Expansion)**
- Persistent log storage for later analysis
- iOS syslog support
- Docker container log integration
- Cloud integration (GCP, AWS, Firebase logging)
- Automated issue detection & insights

## Contributing
We welcome contributions! Please follow these steps:
1. Fork the repo and clone locally
2. Create a new feature branch (`git checkout -b feature-name`)
3. Make your changes and commit (`git commit -m "Added new feature"`)
4. Push to your fork and submit a Pull Request

## License
MIT License © 2024 Adam Deane

## Connect
[![LinkedIn](https://img.shields.io/badge/LinkedIn-Adam_Deane-blue?style=for-the-badge&logo=linkedin)](https://www.linkedin.com/in/adam-deane-93456927/)

### Log Storage
DevInsight now supports persistent log storage with automatic rotation:

```bash
# Save logs to file (default location: ./logs)
cargo run -- --save

# Save logs in interactive mode
cargo run -- -i --save

# Specify custom save location
cargo run -- --save --save-path /path/to/logs

# Set maximum log file size before rotation (in MB)
cargo run -- --save --max-size 200

# Load and analyze previous logs
cargo run -- --load /path/to/logs/logcat_20240321_123456.jsonl
```

Logs are stored in JSONL format with the following features:
- Automatic log rotation based on file size
- Timestamp-based querying
- Device ID tracking
- Full log level and tag preservation

