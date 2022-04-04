use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;

pub enum LogWatcherAction {
    None,
    SeekToEnd,
}

pub struct LogWatcher {
    filename: String,
    unique_id: u64,
    pos: u64,
    reader: BufReader<File>,
    finish: bool,
}

impl LogWatcher {
    pub fn register<P: AsRef<Path>>(filename: P) -> Result<LogWatcher, io::Error> {
        let f = match File::open(&filename) {
            Ok(x) => x,
            Err(err) => return Err(err),
        };

        let metadata = match f.metadata() {
            Ok(x) => x,
            Err(err) => return Err(err),
        };

        let mut reader = BufReader::new(f);
        let pos = metadata.len();
        reader.seek(SeekFrom::Start(pos)).unwrap();
        Ok(LogWatcher {
            filename: filename.as_ref().to_string_lossy().to_string(),
            #[cfg(target_family = "windows")]
            unique_id: metadata.file_index().unwrap(),
            pos: pos,
            reader: reader,
            finish: false,
        })
    }

    pub async fn watch(&mut self) -> Vec<i32> {
        loop {
            let mut line = String::new();
            let resp = self.reader.read_line(&mut line);
            match resp {
                Ok(len) => {
                    if len > 0 {
                        self.pos += len as u64;
                        self.reader.seek(SeekFrom::Start(self.pos)).unwrap();
                        let log_str = line.replace("\n", "");
                        if dota_match_log_message(&log_str) {
                            return fetch_player_ids(&log_str);
                        }
                        line.clear();
                    }
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }
}

fn fetch_player_ids(text: &str) -> Vec<i32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[U:1:\d{8,9}\]").unwrap(); // rust can't use positive or negative lookaheads
    }
    // matches [U:1:370898177]
    let full_ids: Vec<String> = RE
        .find_iter(text)
        .filter_map(|matches| matches.as_str().parse().ok())
        .collect();

    // trims the [U:1:]
    let string_ids = full_ids
        .iter()
        .unique()
        .map(|x| &x[5..x.len() - 1])
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    string_ids
        .iter()
        .map(|id| id.parse::<i32>().unwrap())
        .collect()
}

fn dota_match_log_message(text: &str) -> bool {
    let re = Regex::new(r"Lobby").unwrap();
    re.is_match(text)
}
