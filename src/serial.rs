use std::time::Duration;

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::config::{FlowControlConfig, ParityConfig, SerialConfig};
use crate::error::AppError;

pub fn open_port(config: &SerialConfig) -> Result<Box<dyn SerialPort>, AppError> {
    let port = serialport::new(&config.device, config.baud_rate)
        .data_bits(map_data_bits(config.data_bits)?)
        .parity(map_parity(config.parity))
        .stop_bits(map_stop_bits(config.stop_bits)?)
        .flow_control(map_flow_control(config.flow_control))
        .timeout(Duration::from_millis(config.timeout_ms))
        .open()?;

    Ok(port)
}

fn map_data_bits(bits: u8) -> Result<DataBits, AppError> {
    match bits {
        5 => Ok(DataBits::Five),
        6 => Ok(DataBits::Six),
        7 => Ok(DataBits::Seven),
        8 => Ok(DataBits::Eight),
        _ => Err(AppError::Config(format!(
            "unsupported data_bits '{bits}', expected 5..=8"
        ))),
    }
}

fn map_stop_bits(bits: u8) -> Result<StopBits, AppError> {
    match bits {
        1 => Ok(StopBits::One),
        2 => Ok(StopBits::Two),
        _ => Err(AppError::Config(format!(
            "unsupported stop_bits '{bits}', expected 1 or 2"
        ))),
    }
}

fn map_parity(parity: ParityConfig) -> Parity {
    match parity {
        ParityConfig::None => Parity::None,
        ParityConfig::Odd => Parity::Odd,
        ParityConfig::Even => Parity::Even,
    }
}

fn map_flow_control(flow: FlowControlConfig) -> FlowControl {
    match flow {
        FlowControlConfig::None => FlowControl::None,
        FlowControlConfig::Software => FlowControl::Software,
        FlowControlConfig::Hardware => FlowControl::Hardware,
    }
}
