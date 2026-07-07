pub mod model;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use model::{Alias, AliasEntry};

#[derive(Serialize, Deserialize, Debug, Default)]
struct RawStore {
    #[serde(default)]
    aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Default, Clone)]
pub struct Store {
    pub aliases: HashMap<String, Alias>,
}

impl Store {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Store::default());
        }
        let text = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let raw: RawStore = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        let aliases = raw
            .aliases
            .into_iter()
            .map(|(k, v)| (k, v.into_alias()))
            .collect();
        Ok(Store { aliases })
    }

    /// Atomic save: write to tempfile in same dir, then rename over target.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating dirs for {}", path.display()))?;
        }
        let raw = RawStore {
            aliases: self
                .aliases
                .iter()
                .map(|(k, v)| {
                    let entry = if v.shell.is_none() && v.desc.is_none()
                        && v.tags.is_empty() && v.confirm.is_none()
                    {
                        AliasEntry::Plain(v.cmd.clone())
                    } else {
                        AliasEntry::Full(v.clone())
                    };
                    (k.clone(), entry)
                })
                .collect(),
        };
        let text = toml::to_string_pretty(&raw)
            .context("serializing store")?;

        let dir = path.parent().unwrap_or(Path::new("."));
        let mut tmp = NamedTempFile::new_in(dir).context("creating tempfile")?;
        tmp.write_all(text.as_bytes()).context("writing tempfile")?;
        tmp.flush().context("flushing tempfile")?;
        tmp.persist(path)
            .with_context(|| format!("persisting to {}", path.display()))?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, alias: Alias) {
        self.aliases.insert(name.into(), alias);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trip_plain_alias() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("aliases.toml");

        let mut store = Store::default();
        store.insert("gs", Alias::new("git status"));
        store.save(&path).unwrap();

        let loaded = Store::load(&path).unwrap();
        assert_eq!(loaded.get("gs").unwrap().cmd, "git status");
        assert_eq!(loaded.get("gs").unwrap().shell, None);
    }

    #[test]
    fn round_trip_full_alias() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("aliases.toml");

        let mut store = Store::default();
        store.insert("proj", Alias { cmd: "cd ~/UB".into(), shell: Some(true), desc: Some("go to UB".into()), tags: vec![], confirm: None });
        store.save(&path).unwrap();

        let loaded = Store::load(&path).unwrap();
        let a = loaded.get("proj").unwrap();
        assert_eq!(a.shell, Some(true));
        assert_eq!(a.desc.as_deref(), Some("go to UB"));
    }

    #[test]
    fn hand_written_mixed_file_parses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("aliases.toml");
        fs::write(&path, r#"
[aliases]
gs   = "git status"
proj = { cmd = "cd ~/UB", shell = true }
"#).unwrap();
        let store = Store::load(&path).unwrap();
        assert_eq!(store.get("gs").unwrap().cmd, "git status");
        assert_eq!(store.get("proj").unwrap().shell, Some(true));
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let store = Store::load(&tmp.path().join("no_such.toml")).unwrap();
        assert!(store.aliases.is_empty());
    }

    #[test]
    fn remove_returns_false_when_absent() {
        let mut store = Store::default();
        assert!(!store.remove("nope"));
    }
}
