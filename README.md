# DevInsight: Real-time Log Analyzer for Developers 🚀

![Build Status](https://img.shields.io/github/actions/workflow/status/YOUR_GITHUB/DevInsight/build.yml?branch=main)
![Contributors](https://img.shields.io/github/contributors/YOUR_GITHUB/DevInsight)
![License](https://img.shields.io/github/license/YOUR_GITHUB/DevInsight)

## Overview
**DevInsight** is a blazing-fast, Rust-powered log analysis tool designed for developers who love **log files** and debugging. Starting with **Android logcat**, DevInsight aims to be the go-to tool for real-time log monitoring, filtering, and intelligent insights. Future expansions will include support for iOS syslogs, Docker logs, and cloud-based logging solutions.

## Features (MVP)
✅ **Real-time log streaming** from `adb logcat`
✅ **Custom filtering** (Errors, Warnings, Debug, Specific Tags)
✅ **Regex-based search** for pinpoint debugging
✅ **Color-coded log output** for quick readability
✅ **Optimized Rust performance** for low-latency log processing

## Roadmap
🚀 **Phase 1 (Android Logcat MVP)**
- Basic CLI for streaming logs
- Error and warning filtering
- Custom tag-based search
- Interactive TUI interface using `ratatui`

⚡ **Phase 2 (Advanced Logging & Expansion)**
- Persistent log storage for later analysis
- iOS syslog support
- Docker container log integration
- Cloud integration (GCP, AWS, Firebase logging)
- Automated issue detection & insights

## Installation
### **Ubuntu/Linux**
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

### **Mac (macOS)**
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
```bash
# Start DevInsight
./target/release/devinsight

# Run with custom filters
./target/release/devinsight --filter error --tag SYSTEM
```

## Contribution
We welcome contributions! Please follow these steps:
1. Fork the repo and clone locally.
2. Create a new feature branch (`git checkout -b feature-name`).
3. Make your changes and commit (`git commit -m "Added new feature"`).
4. Push to your fork and submit a Pull Request!

## License
MIT License © 2025 Adam Deane

## Connect
[![LinkedIn](https://img.shields.io/badge/LinkedIn-Adam_Deane-blue?style=for-the-badge&logo=linkedin)](https://www.linkedin.com/in/adam-deane-93456927/)

