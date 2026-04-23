use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::fs_scan::FileSnapshot;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AgentState {
    #[serde(default)]
    pub since: String,
    #[serde(default)]
    pub remote_format: Option<String>,
    #[serde(default)]
    pub files: BTreeMap<String, FileSnapshot>,
}

impl AgentState {
    pub async fn load(path: &Path) -> Result<Self> {
        if !tokio::fs::try_exists(path).await? {
            return Ok(Self::default());
        }
        let raw = tokio::fs::read_to_string(path).await?;
        let state = serde_json::from_str::<Self>(&raw)?;
        Ok(state)
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        ensure_parent(path).await?;
        let raw = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, raw).await?;
        Ok(())
    }
}

async fn ensure_parent(path: &Path) -> Result<()> {
    let parent: PathBuf = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    tokio::fs::create_dir_all(parent).await?;
    Ok(())
}
