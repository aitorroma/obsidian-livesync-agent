use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "livesync-agent",
    version,
    about = "Headless bidirectional vault sync binary"
)]
pub struct Cli {
    /// Path to config TOML.
    #[arg(short, long, default_value = "livesync-agent.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive setup wizard: writes config and validates connection.
    Setup {
        /// Output config path (default: --config value).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Overwrite config file without prompt.
        #[arg(long, default_value_t = false)]
        yes: bool,
        #[arg(long)]
        vault_path: Option<PathBuf>,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        database: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    /// Generate an example config file.
    InitConfig {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run one sync cycle (pull then push).
    SyncOnce,
    /// Run periodic sync loop.
    Daemon {
        #[arg(short, long, default_value_t = 30)]
        interval_seconds: u64,
    },
}
