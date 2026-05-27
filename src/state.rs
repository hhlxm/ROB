use crate::llm::ChatMessage;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub updated_at: u64,
    pub message_count: usize,
}

impl SessionRecord {
    pub fn new(id: String, messages: Vec<ChatMessage>) -> Self {
        let now = now_seconds();
        Self {
            id,
            created_at: now,
            updated_at: now,
            messages,
        }
    }
}

pub fn save_session(id: &str, messages: &[ChatMessage]) -> Result<()> {
    let path = session_path(id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create session dir {}", parent.display()))?;
    }

    let mut record = if path.exists() {
        load_session(id).unwrap_or_else(|_| SessionRecord::new(id.to_string(), Vec::new()))
    } else {
        SessionRecord::new(id.to_string(), Vec::new())
    };
    record.updated_at = now_seconds();
    record.messages = messages.to_vec();

    let raw = serde_json::to_string_pretty(&record)?;
    fs::write(&path, raw).with_context(|| format!("failed to write session {}", path.display()))
}

pub fn load_session(id: &str) -> Result<SessionRecord> {
    let path = session_path(id)?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read session {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse session {}", path.display()))
}

pub fn list_sessions() -> Result<Vec<SessionSummary>> {
    let dir = sessions_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_str::<SessionRecord>(&raw) else {
            continue;
        };
        sessions.push(SessionSummary {
            id: record.id,
            updated_at: record.updated_at,
            message_count: record.messages.len(),
        });
    }

    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(sessions)
}

fn session_path(id: &str) -> Result<PathBuf> {
    if id.trim().is_empty() || id.contains('/') {
        return Err(anyhow!("invalid session id `{id}`"));
    }
    Ok(sessions_dir()?.join(format!("{id}.json")))
}

fn sessions_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("ROB_STATE") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path).join("sessions"));
        }
    }

    let state_dir = dirs::state_dir()
        .or_else(dirs::config_dir)
        .ok_or_else(|| anyhow!("failed to locate state dir"))?;
    Ok(state_dir.join("rob").join("sessions"))
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_path_rejects_path_separators() {
        assert!(session_path("../bad").is_err());
    }
}
