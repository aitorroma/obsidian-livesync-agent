use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::config::AgentConfig;
use crate::couchdb::{AgentFileDoc, CouchDbClient, LiveSyncLeafDoc, LiveSyncPlainDoc};
use crate::fs_scan::{read_file, remove_file, scan_vault, write_file, FileSnapshot};
use crate::state::AgentState;

pub struct SyncEngine {
    cfg: AgentConfig,
    state_path: PathBuf,
    couch: CouchDbClient,
}

impl SyncEngine {
    pub async fn new(cfg: AgentConfig) -> Result<Self> {
        let state_path = cfg
            .state_path
            .clone()
            .unwrap_or_else(|| cfg.vault_path.join(".livesync-agent").join("state.json"));

        let couch = CouchDbClient::new(cfg.couchdb.clone());
        couch.ensure_database_exists().await?;

        Ok(Self {
            cfg,
            state_path,
            couch,
        })
    }

    pub async fn sync_once(&mut self) -> Result<()> {
        info!("sync cycle start");

        let mut state = AgentState::load(&self.state_path).await?;

        self.pull_remote(&mut state).await?;
        self.push_local(&mut state).await?;

        state.save(&self.state_path).await?;
        info!("sync cycle done");

        Ok(())
    }

    async fn pull_remote(&self, state: &mut AgentState) -> Result<()> {
        let changes = self.couch.changes_since(&state.since).await?;

        for row in &changes.results {
            let Some(path) = CouchDbClient::path_from_id(&row.id) else {
                // LiveSync "plain" docs use file path as document id.
                // If not a file:* id, we may still parse based on type.
                if let Some(raw) = row.doc.clone() {
                    let doc_type = raw.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if doc_type == "plain" {
                        state.remote_format = Some("livesync-plain".to_string());
                        let plain: LiveSyncPlainDoc = match serde_json::from_value(raw) {
                            Ok(v) => v,
                            Err(err) => {
                                warn!("skipping malformed plain doc {}: {}", row.id, err);
                                continue;
                            }
                        };
                        self.pull_plain_doc(state, plain).await?;
                    }
                }
                continue;
            };

            let doc = if let Some(raw) = row.doc.clone() {
                match serde_json::from_value::<AgentFileDoc>(raw) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("skipping incompatible doc {}: {}", row.id, err);
                        continue;
                    }
                }
            } else {
                AgentFileDoc {
                    id: row.id.clone(),
                    rev: None,
                    r#type: "agent-file".to_string(),
                    path: path.clone(),
                    deleted: row.deleted,
                    mtime_ms: now_ms(),
                    content_b64: None,
                    sha256: None,
                }
            };

            if doc.r#type != "agent-file" {
                continue;
            }

            if doc.deleted || row.deleted {
                remove_file(&self.cfg.vault_path, &path).await?;
                state.files.remove(&path);
                info!("pulled delete: {}", path);
                continue;
            }

            let should_apply = match state.files.get(&path) {
                Some(local) => doc.mtime_ms > local.mtime_ms,
                None => true,
            };

            if !should_apply {
                continue;
            }

            let bytes = CouchDbClient::decode_content(&doc)?;
            write_file(&self.cfg.vault_path, &path, &bytes).await?;

            let snapshot = snapshot_from_doc(&doc, bytes.len() as u64);
            state.files.insert(path.clone(), snapshot);
            info!("pulled update: {}", path);
        }

        let seq = CouchDbClient::last_seq_to_string(&changes.last_seq);
        state.since = seq;
        Ok(())
    }

    async fn pull_plain_doc(&self, state: &mut AgentState, doc: LiveSyncPlainDoc) -> Result<()> {
        let path = doc.path.clone();
        if doc.deleted {
            remove_file(&self.cfg.vault_path, &path).await?;
            state.files.remove(&path);
            info!("pulled delete(plain): {}", path);
            return Ok(());
        }

        let should_apply = match state.files.get(&path) {
            Some(local) => doc.mtime > local.mtime_ms,
            None => true,
        };
        if !should_apply {
            return Ok(());
        }

        let mut content = String::new();
        let mut incomplete = false;
        for child in &doc.children {
            let Some(v) = self.couch.get_doc_value(child).await? else {
                warn!("plain doc {} missing child leaf {}", doc.path, child);
                incomplete = true;
                continue;
            };
            let leaf: LiveSyncLeafDoc = match serde_json::from_value(v) {
                Ok(v) => v,
                Err(err) => {
                    warn!("plain doc {} invalid child {}: {}", doc.path, child, err);
                    incomplete = true;
                    continue;
                }
            };
            if leaf.r#type != "leaf" {
                warn!(
                    "plain doc {} child {} has unexpected type {}",
                    doc.path, child, leaf.r#type
                );
                incomplete = true;
                continue;
            }
            content.push_str(&leaf.data);
        }
        if incomplete {
            warn!(
                "skipping write for {} because one or more child leaves were incomplete",
                doc.path
            );
            return Ok(());
        }
        let bytes = content.into_bytes();
        write_file(&self.cfg.vault_path, &path, &bytes).await?;
        state.files.insert(
            path.clone(),
            FileSnapshot {
                path,
                mtime_ms: doc.mtime,
                size: bytes.len() as u64,
                sha256: hex_sha256(&bytes),
            },
        );
        info!("pulled update(plain): {}", doc.path);
        Ok(())
    }

    async fn push_local(&self, state: &mut AgentState) -> Result<()> {
        let scanned = scan_vault(&self.cfg.vault_path, &self.cfg.ignore_prefixes).await?;
        let push_as_plain = state.remote_format.as_deref() == Some("livesync-plain");

        let mut known = BTreeSet::new();
        for (path, snap) in &scanned {
            known.insert(path.clone());

            let changed = match state.files.get(path) {
                Some(prev) => prev.sha256 != snap.sha256 || prev.size != snap.size,
                None => true,
            };

            if !changed {
                continue;
            }

            let bytes = read_file(&self.cfg.vault_path, path).await?;
            if push_as_plain {
                self.push_plain_file(path, snap.mtime_ms, &bytes).await?;
            } else {
                let doc = CouchDbClient::doc_from_local(path, snap.mtime_ms, &bytes, &snap.sha256);
                self.couch.put_doc(doc).await?;
            }
            state.files.insert(path.clone(), snap.clone());
            info!("pushed update: {}", path);
        }

        let deleted_paths: Vec<String> = state
            .files
            .keys()
            .filter(|p| !known.contains(*p))
            .cloned()
            .collect();

        for path in deleted_paths {
            let delete_result = if push_as_plain {
                self.couch.delete_doc(&path).await
            } else {
                let tombstone = CouchDbClient::tombstone(&path, now_ms());
                self.couch.put_doc(tombstone).await
            };
            if let Err(err) = delete_result {
                warn!("failed to push tombstone for {}: {err:#}", path);
                continue;
            }
            state.files.remove(&path);
            info!("pushed delete: {}", path);
        }

        debug!("scan_count={}", scanned.len());
        Ok(())
    }

    async fn push_plain_file(&self, path: &str, mtime_ms: i64, bytes: &[u8]) -> Result<()> {
        let text = String::from_utf8_lossy(bytes);
        let chunks = chunk_utf8(&text, 4096);
        let mut children = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let leaf_id = format!("h:{}", short_hash(chunk.as_bytes()));
            let leaf = json!({
                "_id": leaf_id,
                "type": "leaf",
                "data": chunk,
            });
            self.couch.put_doc_value(&leaf_id, leaf).await?;
            children.push(leaf_id);
        }
        let plain = json!({
            "_id": path,
            "type": "plain",
            "path": path,
            "ctime": mtime_ms,
            "mtime": mtime_ms,
            "size": bytes.len() as i64,
            "children": children,
            "eden": {},
        });
        self.couch.put_doc_value(path, plain).await?;
        Ok(())
    }
}

fn snapshot_from_doc(doc: &AgentFileDoc, size: u64) -> FileSnapshot {
    FileSnapshot {
        path: doc.path.clone(),
        mtime_ms: doc.mtime_ms,
        size,
        sha256: doc.sha256.clone().unwrap_or_default(),
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn short_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let h = hasher.finalize();
    h[..8]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn chunk_utf8(input: &str, max_len: usize) -> Vec<String> {
    if input.is_empty() {
        return vec![String::new()];
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < input.len() {
        let mut end = (start + max_len).min(input.len());
        while end < input.len() && !input.is_char_boundary(end) {
            end -= 1;
        }
        if end == start {
            end = input.len();
        }
        out.push(input[start..end].to_string());
        start = end;
    }
    out
}
