use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::error::Error;
use clap::{Arg, Command as ClapCommand};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = ClapCommand::new("DevInsight")
        .version("0.1.0")
        .author("Adam Deane")
        .about("Real-time Android Log Analyzer")
        .arg(Arg::new("filter")
            .short('f')
            .long("filter")
            .num_args(1)
            .help("Filter logs by error level (E, W, D, etc.)"))
        .arg(Arg::new("tag")
            .short('t')
            .long("tag")
            .num_args(1)
            .help("Filter logs by specific tag"))
        .get_matches();

    let filter_level = matches.get_one::<String>("filter");
    let filter_tag = matches.get_one::<String>("tag");

    println!("Starting DevInsight: Real-time Android Log Analyzer...");

    let process = Command::new("adb")
        .arg("logcat")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = process.stdout.ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let log = line?;

        if let Some(level) = filter_level {
            if !log.contains(&format!("{}/", level)) {
                continue;
            }
        }

        if let Some(tag) = filter_tag {
            if !log.contains(tag) {
                continue;
            }
        }

        if log.contains("E/") {
            println!("\x1b[31m[!] {}\x1b[0m", log);
        } else if log.contains("W/") {
            println!("\x1b[33m[!] {}\x1b[0m", log);
        } else {
            println!("{}", log);
        }
    }

    Ok(())
}
