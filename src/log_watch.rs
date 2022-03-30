use std::sync::mpsc::Sender;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::os::windows::fs::MetadataExt;


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

    fn reopen_if_log_rotated(&mut self,  sender: &Sender<String>)
    {
        loop {
            match File::open(&self.filename) {
                Ok(x) => {
                    let f = x;
                    let metadata = match f.metadata() {
                        Ok(m) => m,
                        Err(_) => {
                            sleep(Duration::new(1, 0));
                            continue;
                        }
                    };
                    let current_unique_id = metadata.file_index().unwrap(); 
                    if current_unique_id != self.unique_id {
                        self.finish = true;
                        self.watch(sender);
                        self.finish = false;
                        println!("reloading log file");
                        self.reader = BufReader::new(f);
                        self.pos = 0;
                        self.unique_id = current_unique_id;
                    } else {
                        sleep(Duration::new(1, 0));
                    }
                    break;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        sleep(Duration::new(1, 0));
                        continue;
                    }
                }
            };
        }
    }

    pub fn watch(&mut self, sender: &Sender<String>){
        loop {
            let mut line = String::new();
            let resp = self.reader.read_line(&mut line);
            match resp {
                Ok(len) => {
                    if len > 0 {
                        self.pos += len as u64;
                        self.reader.seek(SeekFrom::Start(self.pos)).unwrap();
                        sender.send(line.replace("\n", ""));
                        line.clear();
                    } else {
                        if self.finish {
                            break;
                        } else {
                            self.reopen_if_log_rotated(sender);
                            self.reader.seek(SeekFrom::Start(self.pos)).unwrap();
                        }
                    }
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }
}