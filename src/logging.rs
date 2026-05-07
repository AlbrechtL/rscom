use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::LoggingConfig;
use crate::error::AppError;

pub struct SessionLogger {
    file: Option<File>,
    timestamps: bool,
}

impl SessionLogger {
    pub fn new(config: &LoggingConfig) -> Result<Self, AppError> {
        let file = match &config.session_log {
            Some(path) => {
                let path_ref = Path::new(path);
                if let Some(parent) = path_ref.parent() {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent)?;
                    }
                }

                Some(
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path_ref)?,
                )
            }
            None => None,
        };

        Ok(Self {
            file,
            timestamps: config.timestamps,
        })
    }

    pub fn log_rx(&mut self, bytes: &[u8]) {
        let _ = self.log("RX", bytes);
    }

    pub fn log_tx(&mut self, bytes: &[u8]) {
        let _ = self.log("TX", bytes);
    }

    fn log(&mut self, dir: &str, bytes: &[u8]) -> Result<(), AppError> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };

        let mut line = String::new();
        if self.timestamps {
            line.push_str(&format!("[{}] ", now_ms()));
        }

        line.push_str(dir);
        line.push(' ');

        for b in bytes {
            line.push_str(&format!("{b:02X} "));
        }

        line.push('|');
        for b in bytes {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            line.push(c);
        }
        line.push('|');
        line.push('\n');

        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
