pub mod cli;
pub mod config;
pub mod error;
pub mod logging;
pub mod macros;
pub mod serial;
pub mod terminal;

use crate::cli::Cli;
use crate::config::AppConfig;
use crate::error::AppError;

pub fn run() -> Result<(), AppError> {
    if !cli::has_user_args() {
        cli::print_help()?;
        return Ok(());
    }

    let cli = Cli::parse_args();
    let mut config = AppConfig::load_from_path(&cli.config)?;
    config.apply_cli_overrides(&cli)?;
    config.validate()?;

    terminal::run(config)
}
