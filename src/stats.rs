use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::state_path;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct UsageEntry {
    pub count: u64,
    pub last_used: u64, // unix seconds
}

#[derive(Serialize, Deserialize, Default)]
pub struct UsageState {
    #[serde(default)]
    pub usage: HashMap<String, UsageEntry>,
}

impl UsageState {
    pub fn load() -> Self {
        let path = state_path();
        if !path.exists() {
            return UsageState::default();
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&text).unwrap_or_default()
    }

    /// Increment run count for `name`. Best-effort — ignores write failures.
    pub fn bump(name: &str) {
        let mut state = Self::load();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = state.usage.entry(name.to_string()).or_default();
        entry.count += 1;
        entry.last_used = now;
        let _ = Self::save_inner(&state);
    }

    fn save_inner(state: &UsageState) -> anyhow::Result<()> {
        let path = state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string(state)?;
        std::fs::write(&path, text)?;
        Ok(())
    }
}
