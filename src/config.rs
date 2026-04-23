use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub vault_path: PathBuf,
    #[serde(default)]
    pub state_path: Option<PathBuf>,
    #[serde(default)]
    pub ignore_prefixes: Vec<String>,
    pub couchdb: CouchDbConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouchDbConfig {
    pub base_url: String,
    pub database: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

impl AgentConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed reading config: {}", path.display()))?;
        let mut cfg: AgentConfig = toml::from_str(&raw)
            .with_context(|| format!("failed parsing TOML config: {}", path.display()))?;

        if cfg.state_path.is_none() {
            cfg.state_path = Some(cfg.vault_path.join(".livesync-agent").join("state.json"));
        }

        Ok(cfg)
    }
}

pub fn write_example_config(path: &Path) -> Result<()> {
    let sample = AgentConfig {
        vault_path: PathBuf::from("/path/to/vault"),
        state_path: Some(PathBuf::from("/path/to/vault/.livesync-agent/state.json")),
        ignore_prefixes: vec![".git/".into(), ".livesync-agent/".into()],
        couchdb: CouchDbConfig {
            base_url: "https://couchdb.example.com".to_string(),
            database: "obsidian-livesync".to_string(),
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
        },
    };

    write_config(path, &sample)
}

pub fn write_config(path: &Path, cfg: &AgentConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string_pretty(cfg)?;
    fs::write(path, toml)?;
    Ok(())
}
