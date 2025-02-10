# DevInsight TODO List 🚀

## ✅ MVP Completed
- [x] Stream real-time logs from `adb logcat`
- [x] Basic CLI with `--filter` and `--tag` options
- [x] Error handling for missing ADB

## 🔥 Next Features

### 1️⃣ Enhanced Filtering
- [ ] Support multiple log levels (`--filter E,W,D`)
- [ ] Regex-based log filtering (`--match "com.example.myapp"`)
- [ ] Exclude logs by keyword (`--exclude "noise"`)

### 2️⃣ Output Formatting
- [ ] Align logs with consistent spacing & colors
- [ ] Add timestamps to log entries
- [ ] Improve readability with structured columns

### 3️⃣ TUI (Terminal UI) with `ratatui`
- [ ] Scrollable, interactive log view
- [ ] Color-code severity dynamically
- [ ] Pause/resume log streaming

### 4️⃣ Log Storage & Analysis
- [ ] Save logs to a file (`--save logs.txt`)
- [ ] Search functionality (`--search "error"`)
- [ ] Basic log statistics (error count, most frequent tags)