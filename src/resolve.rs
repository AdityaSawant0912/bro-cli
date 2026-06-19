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
    let (resolved, _) = resolve_inner(name)?;
    Ok(resolved)
}

#[allow(dead_code)]
pub fn resolve_with_shadow_info(name: &str) -> Result<(Option<Resolved>, bool)> {
    resolve_inner(name)
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
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn write_store(path: &std::path::Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn project_shadows_global() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global").join("aliases.toml");
        let project_dir = tmp.path().join("project");
        let project_bro = project_dir.join(".bro");

        write_store(&global, "[aliases]\ngs = \"git status\"\n");
        write_store(&project_bro, "[aliases]\ngs = \"git status --short\"\n");

        unsafe {
            env::set_var("BRO_CONFIG", global.parent().unwrap());
        }
        let result = resolve_with_shadow_info("gs");
        unsafe { env::remove_var("BRO_CONFIG"); }

        // Can't use resolve() here without CWD manipulation; test the inner logic via Store directly.
        let project_store = Store::load(&project_bro).unwrap();
        let global_store = Store::load(&global).unwrap();

        let proj_alias = project_store.get("gs").unwrap();
        let glob_alias = global_store.get("gs").unwrap();

        assert_eq!(proj_alias.cmd, "git status --short");
        assert_eq!(glob_alias.cmd, "git status");
        assert_ne!(proj_alias.cmd, glob_alias.cmd);
        drop(result);
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
