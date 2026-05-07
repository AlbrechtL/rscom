use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use serialport::ErrorKind;
use serialport::SerialPort;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::logging::SessionLogger;
use crate::macros::MacroSet;
use crate::serial;

pub fn run(config: AppConfig) -> Result<(), AppError> {
    let macros = MacroSet::from_config(&config.macros)?;
    let mut logger = SessionLogger::new(&config.logging)?;

    print_start_banner(&config, &macros);

    let _guard = RawModeGuard::new()?;

    let mut port = match serial::open_port(&config.serial) {
        Ok(port) => {
            print_status(format!(
                "connected to {} @ {}",
                config.serial.device, config.serial.baud_rate
            ));
            Some(port)
        }
        Err(err) => {
            if let AppError::Serial(serial_err) = &err {
                if is_permission_denied(serial_err) {
                    return Err(AppError::Config(permission_denied_message(&config.serial.device)));
                }
            }

            if config.reconnect.enabled {
                print_status(format!(
                    "initial open failed: {err}. reconnect enabled, waiting for device {}",
                    config.serial.device
                ));
                None
            } else {
                return Err(err);
            }
        }
    };

    let mut reconnect_delay = config.reconnect.initial_delay_ms.max(100);
    let mut next_reconnect_try = Instant::now();
    let mut waiting_ctrl_a_command = false;
    let mut stdout = std::io::stdout();

    loop {
        if port.is_none() && config.reconnect.enabled && Instant::now() >= next_reconnect_try {
            match try_reconnect(&config, &mut port) {
                ReconnectOutcome::Connected => {
                    reconnect_delay = config.reconnect.initial_delay_ms.max(100);
                }
                ReconnectOutcome::RetryLater => {
                    next_reconnect_try =
                        Instant::now() + Duration::from_millis(reconnect_delay.max(100));
                    reconnect_delay = (reconnect_delay.saturating_mul(2))
                        .min(config.reconnect.max_delay_ms.max(100));
                }
                ReconnectOutcome::Abort(err) => return Err(err),
            }
        }

        if let Some(serial_port) = port.as_mut() {
            let mut buf = [0u8; 2048];
            match serial_port.read(&mut buf) {
                Ok(size) if size > 0 => {
                    stdout.write_all(&buf[..size])?;
                    stdout.flush()?;
                    logger.log_rx(&buf[..size]);
                }
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {}
                Err(err) => {
                    print_status(format!("serial read error: {err}. disconnecting"));
                    port = None;
                    next_reconnect_try = Instant::now() + Duration::from_millis(reconnect_delay);
                    reconnect_delay = (reconnect_delay.saturating_mul(2)).min(config.reconnect.max_delay_ms.max(100));
                }
            }
        }

        if event::poll(Duration::from_millis(20))? {
            let Event::Key(key_event) = event::read()? else {
                continue;
            };

            if waiting_ctrl_a_command {
                waiting_ctrl_a_command = false;
                if matches!(key_event.code, KeyCode::Char('x') | KeyCode::Char('X')) {
                    print_status("exit requested via Ctrl+A x");
                    break;
                }
                print_status("unknown Ctrl+A command (use x to exit)");
                continue;
            }

            if key_event.modifiers.contains(KeyModifiers::CONTROL)
                && key_event.code == KeyCode::Char('a')
            {
                waiting_ctrl_a_command = true;
                print_status("Ctrl+A command mode: x=exit");
                continue;
            }

            if let Some(binding) = macros.binding_for_keycode(&key_event.code) {
                let mut payload = binding.command.clone().into_bytes();
                payload.extend_from_slice(b"\r\n");
                write_to_port(&mut port, &payload, &mut logger);
                print_status(format!("macro sent: {}", binding.command));
                continue;
            }

            if let Some(payload) = key_event_to_serial_bytes(key_event) {
                write_to_port(&mut port, &payload, &mut logger);
            }
        }
    }

    Ok(())
}

fn print_start_banner(config: &AppConfig, macros: &MacroSet) {
    eprintln!("rscom: serial terminal for embedded development");
    eprintln!(
        "device={} baud={} leader=Ctrl+A reconnect={}",
        config.serial.device,
        config.serial.baud_rate,
        if config.reconnect.enabled { "on" } else { "off" }
    );

    for (key, binding) in macros.list() {
        if let Some(desc) = &binding.description {
            eprintln!("macro F{key}: {} ({desc})", binding.command);
        } else {
            eprintln!("macro F{key}: {}", binding.command);
        }
    }
}

fn key_event_to_serial_bytes(event: KeyEvent) -> Option<Vec<u8>> {
    match event.code {
        KeyCode::Char(c) => {
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                let lower = c.to_ascii_lowercase();
                if lower.is_ascii_lowercase() {
                    return Some(vec![(lower as u8) - b'a' + 1]);
                }
                return None;
            }

            Some(c.to_string().into_bytes())
        }
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x08]),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Esc => Some(vec![0x1B]),
        _ => None,
    }
}

fn write_to_port(
    port: &mut Option<Box<dyn SerialPort>>,
    payload: &[u8],
    logger: &mut SessionLogger,
) {
    let Some(serial_port) = port.as_mut() else {
        print_status("not connected: cannot send data");
        return;
    };

    match serial_port.write_all(payload) {
        Ok(()) => logger.log_tx(payload),
        Err(err) => {
            print_status(format!("serial write error: {err}. disconnecting"));
            *port = None;
        }
    }
}

fn try_reconnect(config: &AppConfig, port: &mut Option<Box<dyn SerialPort>>) -> ReconnectOutcome {
    match serial::open_port(&config.serial) {
        Ok(new_port) => {
            *port = Some(new_port);
            print_status(format!(
                "reconnected to {} @ {}",
                config.serial.device, config.serial.baud_rate
            ));
            ReconnectOutcome::Connected
        }
        Err(err) => {
            if let AppError::Serial(serial_err) = &err {
                if is_permission_denied(serial_err) {
                    return ReconnectOutcome::Abort(AppError::Config(permission_denied_message(
                        &config.serial.device,
                    )));
                }
            }

            print_status(format!("reconnect failed: {err}"));
            ReconnectOutcome::RetryLater
        }
    }
}

fn print_status(message: impl AsRef<str>) {
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(b"\r\n");
    let _ = stderr.write_all(message.as_ref().as_bytes());
    let _ = stderr.write_all(b"\r\n");
    let _ = stderr.flush();
}

fn is_permission_denied(err: &serialport::Error) -> bool {
    matches!(err.kind(), ErrorKind::Io(std::io::ErrorKind::PermissionDenied))
}

fn permission_denied_message(device: &str) -> String {
    format!(
        "cannot open {device}: permission denied. Add your user to the serial-device group (usually 'dialout') with 'sudo usermod -aG dialout $USER' and start a new login session"
    )
}

enum ReconnectOutcome {
    Connected,
    RetryLater,
    Abort(AppError),
}

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Result<Self, AppError> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
