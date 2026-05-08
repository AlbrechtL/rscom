use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::cli::Cli;
use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub reconnect: ReconnectConfig,
    pub macros: Vec<MacroBindingConfig>,
    pub logging: LoggingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            serial: SerialConfig::default(),
            reconnect: ReconnectConfig::default(),
            macros: Vec::new(),
            logging: LoggingConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let raw = fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&raw)?;
        Ok(cfg)
    }

    pub fn apply_cli_overrides(&mut self, cli: &Cli) -> Result<(), AppError> {
        if let Some(device) = &cli.device {
            if device.trim().is_empty() {
                return Err(AppError::Config("device override must not be empty".to_string()));
            }
            self.serial.device = device.clone();
        }

        if let Some(baud) = cli.baud {
            self.serial.baud_rate = baud;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.serial.device.trim().is_empty() {
            return Err(AppError::Config("serial.device must not be empty".to_string()));
        }

        if self.serial.baud_rate == 0 {
            return Err(AppError::Config(
                "serial.baud_rate must be greater than zero".to_string(),
            ));
        }

        if !(5..=8).contains(&self.serial.data_bits) {
            return Err(AppError::Config(
                "serial.data_bits must be in range 5..=8".to_string(),
            ));
        }

        if !(self.serial.stop_bits == 1 || self.serial.stop_bits == 2) {
            return Err(AppError::Config(
                "serial.stop_bits must be either 1 or 2".to_string(),
            ));
        }

        for item in &self.macros {
            if item.char_delay_ms.is_some() && item.echo_timeout_ms.is_some() {
                return Err(AppError::Config(format!(
                    "macro {}: char_delay_ms and echo_timeout_ms are mutually exclusive",
                    item.key
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SerialConfig {
    pub device: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: ParityConfig,
    pub stop_bits: u8,
    pub flow_control: FlowControlConfig,
    pub timeout_ms: u64,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            device: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            data_bits: 8,
            parity: ParityConfig::None,
            stop_bits: 1,
            flow_control: FlowControlConfig::None,
            timeout_ms: 30,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParityConfig {
    None,
    Odd,
    Even,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowControlConfig {
    None,
    Software,
    Hardware,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ReconnectConfig {
    pub enabled: bool,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay_ms: 500,
            max_delay_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub session_log: Option<String>,
    pub timestamps: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            session_log: None,
            timestamps: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MacroBindingConfig {
    pub key: String,
    pub command: String,
    pub description: Option<String>,
    /// Fixed inter-character delay in milliseconds. Mutually exclusive with `echo_timeout_ms`.
    pub char_delay_ms: Option<u64>,
    /// Per-character echo-sync timeout in milliseconds. Each character is held until the device
    /// echoes it back, or this timeout elapses. Mutually exclusive with `char_delay_ms`.
    pub echo_timeout_ms: Option<u64>,
}
