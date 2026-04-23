mod cli;
mod config;
mod couchdb;
mod fs_scan;
mod setup;
mod state;
mod sync_engine;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

use crate::cli::{Cli, Commands};
use crate::config::AgentConfig;
use crate::setup::SetupParams;
use crate::sync_engine::SyncEngine;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "livesync_agent=info,info".to_string()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Setup {
            output,
            yes,
            vault_path,
            base_url,
            database,
            username,
            password,
        } => {
            setup::run_setup(
                SetupParams {
                    output,
                    yes,
                    vault_path,
                    base_url,
                    database,
                    username,
                    password,
                },
                cli.config.clone(),
            )
            .await
        }
        Commands::InitConfig { output } => {
            let path = output.unwrap_or_else(|| PathBuf::from("livesync-agent.toml"));
            config::write_example_config(&path)?;
            info!("Example config written to {}", path.display());
            Ok(())
        }
        Commands::SyncOnce => {
            let config = AgentConfig::load(&cli.config)?;
            let mut engine = SyncEngine::new(config).await?;
            engine.sync_once().await
        }
        Commands::Daemon { interval_seconds } => {
            let config = AgentConfig::load(&cli.config)?;
            let mut engine = SyncEngine::new(config).await?;
            let tick = Duration::from_secs(interval_seconds.max(5));
            info!("Starting daemon loop with interval {:?}", tick);
            loop {
                if let Err(e) = engine.sync_once().await {
                    error!("sync failed: {e:#}");
                }
                tokio::time::sleep(tick).await;
            }
        }
    }
}
