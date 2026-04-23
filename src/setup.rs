use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::{self, AgentConfig, CouchDbConfig};
use crate::couchdb::CouchDbClient;

#[derive(Debug, Clone)]
pub struct SetupParams {
    pub output: Option<PathBuf>,
    pub yes: bool,
    pub vault_path: Option<PathBuf>,
    pub base_url: Option<String>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub async fn run_setup(params: SetupParams, default_config: PathBuf) -> Result<()> {
    let output = params.output.unwrap_or(default_config);

    if output.exists() && !params.yes {
        let overwrite = prompt_yes_no(
            &format!("Config already exists at {}. Overwrite?", output.display()),
            false,
        )?;
        if !overwrite {
            info!("Setup cancelled.");
            return Ok(());
        }
    }

    let vault_path = match params.vault_path {
        Some(v) => v,
        None => PathBuf::from(prompt_required("Vault path", "/home/user/Obsidian")?),
    };

    let base_url = match params.base_url {
        Some(v) => v,
        None => prompt_required("CouchDB base URL", "https://couchdb.example.com")?,
    };

    let database = match params.database {
        Some(v) => v,
        None => prompt_required("CouchDB database", "obsidian")?,
    };

    let username = match params.username {
        Some(v) => Some(v),
        None => {
            let v = prompt_optional("CouchDB username (empty for none)")?;
            if v.trim().is_empty() {
                None
            } else {
                Some(v)
            }
        }
    };

    let password = match params.password {
        Some(v) => Some(v),
        None => {
            if username.is_some() {
                let p = rpassword::prompt_password("CouchDB password (hidden): ")?;
                Some(p)
            } else {
                None
            }
        }
    };

    let cfg = AgentConfig {
        vault_path: vault_path.clone(),
        state_path: Some(vault_path.join(".livesync-agent").join("state.json")),
        ignore_prefixes: vec![".git/".into(), ".livesync-agent/".into()],
        couchdb: CouchDbConfig {
            base_url,
            database,
            username,
            password,
        },
    };

    config::write_config(&output, &cfg)?;
    info!("Config written to {}", output.display());

    let client = CouchDbClient::new(cfg.couchdb.clone());
    client
        .changes_since("0")
        .await
        .with_context(|| "connection test failed (_changes request)")?;

    info!("Connection check: OK");
    info!(
        "Next: livesync-agent --config {} sync-once",
        output.display()
    );

    Ok(())
}

fn prompt_required(label: &str, example: &str) -> Result<String> {
    loop {
        print!("{} [{}]: ", label, example);
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let value = buf.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
        println!("Value required.");
    }
}

fn prompt_optional(label: &str) -> Result<String> {
    print!("{}: ", label);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_yes_no(label: &str, default_yes: bool) -> Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", label, suffix);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let v = buf.trim().to_lowercase();
    if v.is_empty() {
        return Ok(default_yes);
    }
    Ok(matches!(v.as_str(), "y" | "yes"))
}
