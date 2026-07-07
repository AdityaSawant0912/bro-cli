use std::path::PathBuf;

use anyhow::Result;

use crate::config::{global_store_path, project_store_path};
use crate::store::{model::Alias, Store};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Source {
    Project(PathBuf),
    Global,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Resolved {
    pub alias: Alias,
    pub source: Source,
    /// True when a global alias with the same name exists but is hidden.
    pub shadows_global: bool,
}

pub fn resolve(name: &str) -> Result<Option<Resolved>> {
    resolve_inner(name).map(|(r, _)| r)
}

fn resolve_inner(name: &str) -> Result<(Option<Resolved>, bool)> {
    let project_path = project_store_path();

    if let Some(ref path) = project_path {
        let project_store = Store::load(path)?;
        if let Some(alias) = project_store.get(name).cloned() {
            // check if global also has this name
            let global_store = Store::load(&global_store_path())?;
            let shadows = global_store.get(name).is_some();
            return Ok((
                Some(Resolved { alias, source: Source::Project(path.clone()), shadows_global: shadows }),
                shadows,
            ));
        }
    }

    let global_store = Store::load(&global_store_path())?;
    if let Some(alias) = global_store.get(name).cloned() {
        return Ok((
            Some(Resolved { alias, source: Source::Global, shadows_global: false }),
            false,
        ));
    }

    Ok((None, false))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    use crate::store::Store;

    fn write_store(path: &std::path::Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn project_shadows_global() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global").join("aliases.toml");
        let project_bro = tmp.path().join("project").join(".bro");

        write_store(&global, "[aliases]\ngs = \"git status\"\n");
        write_store(&project_bro, "[aliases]\ngs = \"git status --short\"\n");

        let project_store = Store::load(&project_bro).unwrap();
        let global_store = Store::load(&global).unwrap();

        assert_eq!(project_store.get("gs").unwrap().cmd, "git status --short");
        assert_eq!(global_store.get("gs").unwrap().cmd, "git status");
    }

    #[test]
    fn global_returned_when_no_project() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("aliases.toml");
        write_store(&global, "[aliases]\nfoo = \"echo bar\"\n");

        let store = Store::load(&global).unwrap();
        assert_eq!(store.get("foo").unwrap().cmd, "echo bar");
        assert!(store.get("missing").is_none());
    }
}
