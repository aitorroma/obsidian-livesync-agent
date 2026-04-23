use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: String,
    pub mtime_ms: i64,
    pub size: u64,
    pub sha256: String,
}

pub async fn scan_vault(vault_root: &Path, ignore_prefixes: &[String]) -> Result<BTreeMap<String, FileSnapshot>> {
    let root = vault_root.canonicalize().with_context(|| {
        format!("failed to resolve vault path: {}", vault_root.display())
    })?;

    let mut files = BTreeMap::new();

    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let abs = entry.path().to_path_buf();
        let rel = relative_path(&root, &abs)?;

        if should_ignore(&rel, ignore_prefixes) {
            continue;
        }

        let metadata = entry.metadata()?;
        let mtime_ms = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let bytes = tokio::fs::read(&abs).await?;
        let sha256 = hex_sha256(&bytes);

        files.insert(
            rel.clone(),
            FileSnapshot {
                path: rel,
                mtime_ms,
                size: metadata.len(),
                sha256,
            },
        );
    }

    Ok(files)
}

pub async fn read_file(vault_root: &Path, rel_path: &str) -> Result<Vec<u8>> {
    let abs = vault_root.join(rel_path);
    let bytes = tokio::fs::read(abs).await?;
    Ok(bytes)
}

pub async fn write_file(vault_root: &Path, rel_path: &str, bytes: &[u8]) -> Result<()> {
    let abs = vault_root.join(rel_path);
    if let Some(parent) = abs.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(abs, bytes).await?;
    Ok(())
}

pub async fn remove_file(vault_root: &Path, rel_path: &str) -> Result<()> {
    let abs = vault_root.join(rel_path);
    if tokio::fs::try_exists(&abs).await? {
        tokio::fs::remove_file(abs).await?;
    }
    Ok(())
}

fn relative_path(root: &Path, abs: &PathBuf) -> Result<String> {
    let rel = abs
        .strip_prefix(root)
        .with_context(|| format!("file not inside vault: {}", abs.display()))?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(rel)
}

fn should_ignore(rel: &str, ignore_prefixes: &[String]) -> bool {
    ignore_prefixes.iter().any(|p| rel.starts_with(p))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
