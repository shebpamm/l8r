mod haproxy;
mod utils;

use crate::haproxy::HaproxyLogEntry;
use crate::utils::{is_stdin_redirected, output_table, reset_sigpipe};
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;
use clap::Parser;
use regex::Regex;
use once_cell::sync::Lazy;
use serde::Serialize;
use anyhow::Result;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;

static RE: Lazy<Regex> = regex_static::lazy_regex!(r#"^(?P<month>[A-Za-z]{3})\s+(?P<day>\d{1,2})\s+(?P<time>[0-9:]{8})\s+(?P<host>\w+)\s+(?P<process_id>[A-Za-z0-9]+\[\d+\]):\s+(?P<source_ip_port>[0-9.]+:[0-9]+)\s+\[(?P<time_stamp_accepted>.+)\]\s+(?P<frontend_name>\w+)\s+(?P<backend_name>[\w-]+)/(?P<server_name>[-\w]+)\s+(?P<queues_stats>\d+/\d+/\d+/\d+/\d+)\s+(?P<response_code>\d+)\s+(?P<bytes_read>\d+)\s-\s-\s(?P<termination_state>[-\w]{4})\s(?P<conn_counts>\d+/\d+/\d+/\d+/\d+)\s+(?P<queue>\d+/\d+)\s+"(?P<request>.*)"$"#);

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    Raw,
    #[default]
    Color,
    Json,
    Yaml,
    Wide,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    pub file: Option<PathBuf>,
    #[arg(short, long)]
    pub errors: bool,
    #[arg(short, long)]
    pub terminations: bool,
    #[arg(short, long)]
    pub matcher: Option<String>,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub output: Option<OutputFormat>,
    #[arg(long)]
    #[clap(default_value = "false")]
    pub serial: bool
}

enum Reader {
    File(BufReader<File>),
    Stdin(BufReader<std::io::Stdin>),
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    reset_sigpipe();
    let args = Args::parse();

    let matcher: Option<Regex> = match args.matcher {
        Some(m) => Some(Regex::new(&m)?),
        None => None
    };

    let reader: Reader = match &args.file {
        Some(file) => {
            let file = File::open(file)?;
            let reader = BufReader::new(file);
            Reader::File(reader)
        }
        None => {
            if is_stdin_redirected()? {
                let reader = BufReader::new(std::io::stdin());
                Reader::Stdin(reader)
            } else {
                return Err("No input provided".into());
            }
        }
    };

        let parser = |line: String| {
            if let Some(ref matcher) = matcher {
                if !matcher.is_match(&line) {
                    return
                }
            }

            match HaproxyLogEntry::parse(&line) {
                Ok(entry) => {
                    if args.errors && !entry.is_error() {
                        return
                    }

                    if args.terminations && !entry.termination_state.is_error() {
                        return
                    }

                    println!("{}", match args.output {
                        Some(OutputFormat::Raw) => entry.colorless(),
                        Some(OutputFormat::Json) => serde_json::to_string(&entry).unwrap(),
                        Some(OutputFormat::Yaml) => { 
                            format!("---\n{}",
                                serde_yaml::to_string(&entry).unwrap()
                            )
                        }
                        Some(OutputFormat::Wide) => output_table(&entry).unwrap(),
                        Some(OutputFormat::Color) | None => entry.colorize()
                    });
                }
                Err(_) => {
                    if args.verbose {
                        eprintln!("Failed to parse line: {}", line);
                    }
                }
            }
    };

    match reader {
        Reader::File(reader) => {
            if args.serial {
                reader.lines().filter_map(|line| line.ok()).for_each(parser);
            } else {
                reader.lines().par_bridge().filter_map(|line| line.ok()).for_each(parser);
            }
        }
        Reader::Stdin(reader) => {
            if args.serial {
                reader.lines().filter_map(|line| line.ok()).for_each(parser);
            } else {
                reader.lines().par_bridge().filter_map(|line| line.ok()).for_each(parser);
            }
        }
    
    }
    Ok(())
}
