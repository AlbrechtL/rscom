use clap::{CommandFactory, Parser};

#[derive(Debug, Clone, Parser)]
#[command(name = "rscom", version, about = "Rust serial console for embedded development")]
pub struct Cli {
    /// Path to YAML configuration file
    #[arg(short, long, default_value = "rscom.yaml")]
    pub config: String,

    /// Override serial device (for example /dev/ttyUSB0)
    #[arg(short = 'd', long)]
    pub device: Option<String>,

    /// Override baud rate (for example 115200)
    #[arg(short = 'b', long)]
    pub baud: Option<u32>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub fn has_user_args() -> bool {
    std::env::args_os().len() > 1
}

pub fn print_help() -> std::io::Result<()> {
    let mut cmd = Cli::command();
    cmd.print_help()?;
    eprintln!();
    Ok(())
}
