use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

use crate::config::CouchDbConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default)]
    pub rev: Option<String>,
    #[serde(default = "default_type")]
    pub r#type: String,
    pub path: String,
    #[serde(default)]
    pub deleted: bool,
    pub mtime_ms: i64,
    #[serde(default)]
    pub content_b64: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSyncPlainDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default)]
    pub rev: Option<String>,
    #[serde(default = "default_plain_type")]
    pub r#type: String,
    pub path: String,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub ctime: i64,
    #[serde(default)]
    pub mtime: i64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub eden: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSyncLeafDoc {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default)]
    pub rev: Option<String>,
    #[serde(default = "default_leaf_type")]
    pub r#type: String,
    pub data: String,
}

fn default_type() -> String {
    "agent-file".to_string()
}
fn default_plain_type() -> String {
    "plain".to_string()
}
fn default_leaf_type() -> String {
    "leaf".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ChangesResponse {
    #[serde(default)]
    pub results: Vec<ChangeRow>,
    #[serde(default)]
    pub last_seq: Value,
}

#[derive(Debug, Deserialize)]
pub struct ChangeRow {
    pub id: String,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub doc: Option<Value>,
}

#[derive(Clone)]
pub struct CouchDbClient {
    cfg: CouchDbConfig,
    http: reqwest::Client,
}

impl CouchDbClient {
    pub fn new(cfg: CouchDbConfig) -> Self {
        let http = reqwest::Client::builder().build().expect("reqwest client");
        Self { cfg, http }
    }

    pub async fn ensure_database_exists(&self) -> Result<()> {
        let url = self.db_url();
        let req = self.auth(self.http.get(&url));
        let res = req.send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            let create = self.auth(self.http.put(&url)).send().await?;
            if !create.status().is_success() && create.status() != StatusCode::PRECONDITION_FAILED {
                return Err(anyhow!("failed to create database: {}", create.status()));
            }
            return Ok(());
        }
        if !res.status().is_success() {
            return Err(anyhow!("failed to check database: {}", res.status()));
        }
        Ok(())
    }

    pub async fn changes_since(&self, since: &str) -> Result<ChangesResponse> {
        let url = format!(
            "{}/_changes?include_docs=true&since={}&limit=10000",
            self.db_url(),
            urlencoding::encode(if since.is_empty() { "0" } else { since })
        );
        let res = self.auth(self.http.get(url)).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("_changes failed: {}", res.status()));
        }
        Ok(res.json::<ChangesResponse>().await?)
    }

    pub async fn get_doc(&self, id: &str) -> Result<Option<AgentFileDoc>> {
        match self.get_doc_value(id).await? {
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
            None => Ok(None),
        }
    }

    pub async fn get_doc_value(&self, id: &str) -> Result<Option<Value>> {
        let url = format!("{}/{}", self.db_url(), urlencoding::encode(id));
        let res = self.auth(self.http.get(url)).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !res.status().is_success() {
            return Err(anyhow!("get doc failed: {}", res.status()));
        }
        Ok(Some(res.json::<Value>().await?))
    }

    pub async fn put_doc(&self, mut doc: AgentFileDoc) -> Result<()> {
        let existing = self.get_doc(&doc.id).await?;
        if let Some(old) = existing {
            doc.rev = old.rev;
        }

        let url = format!("{}/{}", self.db_url(), urlencoding::encode(&doc.id));
        let res = self.auth(self.http.put(url).json(&doc)).send().await?;
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("put doc failed: {} body={}", status, body));
        }
        Ok(())
    }

    pub async fn put_doc_value(&self, id: &str, mut doc: Value) -> Result<()> {
        if let Some(existing) = self.get_doc_value(id).await? {
            if let Some(rev) = existing.get("_rev").and_then(|v| v.as_str()) {
                doc["_rev"] = Value::String(rev.to_string());
            }
        }
        let url = format!("{}/{}", self.db_url(), urlencoding::encode(id));
        let res = self.auth(self.http.put(url).json(&doc)).send().await?;
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("put json doc failed: {} body={}", status, body));
        }
        Ok(())
    }

    pub async fn delete_doc(&self, id: &str) -> Result<()> {
        let Some(existing) = self.get_doc_value(id).await? else {
            return Ok(());
        };
        let Some(rev) = existing.get("_rev").and_then(|v| v.as_str()) else {
            return Ok(());
        };
        let mut tombstone = serde_json::Map::new();
        tombstone.insert("_id".to_string(), Value::String(id.to_string()));
        tombstone.insert("_rev".to_string(), Value::String(rev.to_string()));
        tombstone.insert("_deleted".to_string(), Value::Bool(true));
        let url = format!("{}/{}", self.db_url(), urlencoding::encode(id));
        let res = self
            .auth(self.http.put(url).json(&Value::Object(tombstone)))
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(anyhow!("delete doc failed: {}", res.status()));
        }
        Ok(())
    }

    pub fn id_for_path(path: &str) -> String {
        format!("file:{}", path)
    }

    pub fn path_from_id(id: &str) -> Option<String> {
        id.strip_prefix("file:").map(ToString::to_string)
    }

    fn db_url(&self) -> String {
        format!(
            "{}/{}",
            self.cfg.base_url.trim_end_matches('/'),
            self.cfg.database
        )
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match (&self.cfg.username, &self.cfg.password) {
            (Some(user), Some(pass)) => req.basic_auth(user, Some(pass)),
            _ => req,
        }
    }

    pub fn doc_from_local(path: &str, mtime_ms: i64, bytes: &[u8], sha256: &str) -> AgentFileDoc {
        AgentFileDoc {
            id: Self::id_for_path(path),
            rev: None,
            r#type: default_type(),
            path: path.to_string(),
            deleted: false,
            mtime_ms,
            content_b64: Some(B64.encode(bytes)),
            sha256: Some(sha256.to_string()),
        }
    }

    pub fn tombstone(path: &str, mtime_ms: i64) -> AgentFileDoc {
        AgentFileDoc {
            id: Self::id_for_path(path),
            rev: None,
            r#type: default_type(),
            path: path.to_string(),
            deleted: true,
            mtime_ms,
            content_b64: None,
            sha256: None,
        }
    }

    pub fn decode_content(doc: &AgentFileDoc) -> Result<Vec<u8>> {
        let content = doc
            .content_b64
            .as_ref()
            .ok_or_else(|| anyhow!("missing content_b64 for {}", doc.path))?;
        Ok(B64.decode(content)?)
    }

    pub fn last_seq_to_string(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            _ => json!(v).to_string(),
        }
    }
}
