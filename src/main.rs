use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;
use clap::Parser;
use regex::Regex;
use once_cell::sync::Lazy;
use colored::Colorize;
use anyhow::Error;
use atty::Stream;

static RE: Lazy<Regex> = regex_static::lazy_regex!(r#"^(?P<month>[A-Za-z]{3})\s+(?P<day>\d{1,2})\s+(?P<time>[0-9:]{8})\s+(?P<host>\w+)\s+(?P<process_id>[A-Za-z0-9]+\[\d+\]):\s+(?P<source_ip_port>[0-9.]+:[0-9]+)\s+\[(?P<time_stamp_accepted>.+)\]\s+(?P<frontend_name>\w+)\s+(?P<backend_name>[\w-]+)/(?P<server_name>[-\w]+)\s+(?P<queues_stats>\d+/\d+/\d+/\d+/\d+)\s+(?P<response_code>\d+)\s+(?P<bytes_read>\d+)\s-\s-\s(?P<termination_state>[-\w]{4})\s(?P<conn_counts>\d+/\d+/\d+/\d+/\d+)\s+(?P<queue>\d+/\d+)\s+"(?P<request>.*)"$"#);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    pub file: Option<PathBuf>,
    #[arg(short, long)]
    pub dull: bool,
    #[arg(short, long)]
    pub errors: bool,
    #[arg(short, long)]
    pub matcher: Option<String>,
    #[arg(short, long)]
    pub verbose: bool,
}

// May  8 00:08:30 applb05 haproxy[3091252]: 127.0.0.1:6102 [08/May/2024:00:08:30.660] mclbfe silo-mclb-silo-backend/kube-prod2-node16 0/0/9/17/26 200 1005 - - ---- 823/541/29/2/0 0/0 "GET /silo/collections/1b629de5_1aaf_47d7_8b6d_5cfdcc8337e3 HTTP/1.1"
#[derive(Debug)]
pub struct HaproxyLogEntry {
    pub month: String,
    pub day: String,
    pub time: String,
    pub host: String,
    pub process_id: String,
    pub source_ip_port: String, 
    pub time_stamp_accepted: String,
    pub frontend_name: String, 
    pub backend_name: String, 
    pub server_name: String,
    pub queues_stats: String,
    pub response_code: String,
    pub bytes_read: String,
    pub termination_state: String,
    pub conn_counts: String,
    pub queue: String,
    pub request: String, 
}

impl HaproxyLogEntry {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let captures = RE.captures(s).ok_or("Failed to parse line")?;
        let data = HaproxyLogEntry {
            month: captures.name("month").ok_or("")?.as_str().to_string(),
            day: captures.name("day").ok_or("")?.as_str().to_string(),
            time: captures.name("time").ok_or("")?.as_str().to_string(),
            host: captures.name("host").ok_or("")?.as_str().to_string(),
            process_id: captures.name("process_id").ok_or("")?.as_str().to_string(),
            source_ip_port: captures.name("source_ip_port").ok_or("")?.as_str().to_string(),
            time_stamp_accepted: captures.name("time_stamp_accepted").ok_or("")?.as_str().to_string(),
            frontend_name: captures.name("frontend_name").ok_or("")?.as_str().to_string(),
            backend_name: captures.name("backend_name").ok_or("")?.as_str().to_string(),
            server_name: captures.name("server_name").ok_or("")?.as_str().to_string(),
            queues_stats: captures.name("queues_stats").ok_or("")?.as_str().to_string(),
            response_code: captures.name("response_code").ok_or("")?.as_str().to_string(),
            bytes_read: captures.name("bytes_read").ok_or("")?.as_str().to_string(),
            termination_state: captures.name("termination_state").ok_or("")?.as_str().to_string(),
            conn_counts: captures.name("conn_counts").ok_or("")?.as_str().to_string(),
            queue: captures.name("queue").ok_or("")?.as_str().to_string(),
            request: captures.name("request").ok_or("")?.as_str().to_string(),
        };

        Ok(data)
    }

    fn colorless(&self) -> String {
        format!("{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.month,
            self.day,
            self.time,
            self.host,
            self.process_id,
            self.source_ip_port,
            self.time_stamp_accepted,
            self.frontend_name,
            self.backend_name,
            self.server_name,
            self.queues_stats,
            self.response_code,
            self.bytes_read,
            self.termination_state,
            self.conn_counts,
            self.queue,
            self.request
        )
    }
    fn colorize(&self) -> String {
        format!("{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.month.white(),
            self.day.white(),
            self.time.white(),
            self.host.white(),
            self.process_id.white(),
            self.source_ip_port.white(),
            self.time_stamp_accepted.white(),
            self.frontend_name.purple(),
            self.backend_name.yellow(),
            self.server_name.blue(),
            self.queues_stats.white(),
            match self.response_code.as_str().parse::<u16>() {
                Ok(code) => {
                    if code >= 200 && code < 300 {
                        self.response_code.green()
                    } else if code >= 300 && code < 400 {
                        self.response_code.yellow()
                    } else if code >= 400 {
                        self.response_code.red()
                    } else {
                        self.response_code.white()
                    }
                }
                Err(_) => self.response_code.white()
            },
            self.bytes_read.white(),
            match self.termination_state.as_str() {
                "----" => self.termination_state.green(),
                _ => self.termination_state.red()
            },
            self.conn_counts.white(),
            self.queue.white(),
            self.request.white()
        )

    }

    // Check if error code is 400 or higher, or if no ---- termination_state
    fn is_error(&self) -> bool {
        match self.response_code.as_str().parse::<u16>() {
            Ok(code) => code >= 400 || self.termination_state != "----",
            Err(_) => true
        }
    
    }


}

fn is_stdin_redirected() -> Result<bool, Error> {
    if atty::is(Stream::Stdin) {
        return Ok(false);
    }

    Ok(true)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let matcher: Option<Regex> = match args.matcher {
        Some(m) => Some(Regex::new(&m)?),
        None => None
    };

    let reader: Box<dyn BufRead> = match &args.file {
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
        None => {
            if is_stdin_redirected()? {
                Box::new(BufReader::new(std::io::stdin()))
            } else {
                return Err("No input provided".into());
            }
        }
    };

    for line in reader.lines() {
        let line = line?;

        if let Some(ref matcher) = matcher {
            if !matcher.is_match(&line) {
                continue;
            }
        }

        match HaproxyLogEntry::parse(&line) {
            Ok(entry) => {
                if args.errors && !entry.is_error() {
                    continue;
                }

                println!("{}", match args.dull {
                    true => entry.colorless(),
                    false => entry.colorize()
                });
            }
            Err(_) => {
                if args.verbose {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
    }
    Ok(())
}
