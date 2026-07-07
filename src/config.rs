use std::env;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "bro")
}

/// Global alias store path.
/// Resolution order:
///   1. $BRO_CONFIG env var (file → use as-is; dir → <dir>/aliases.toml)
///   2. Platform default (~/.config/bro/aliases.toml etc.)
pub fn global_store_path() -> PathBuf {
    if let Ok(val) = env::var("BRO_CONFIG") {
        let p = PathBuf::from(&val);
        if p.is_file() {
            return p;
        }
        // treat as directory (whether it exists yet or not)
        return p.join("aliases.toml");
    }
    project_dirs()
        .map(|d| d.config_dir().join("aliases.toml"))
        .unwrap_or_else(|| PathBuf::from("aliases.toml"))
}

/// Reserved path for mutable run-time state (usage counts etc.).
pub fn state_path() -> PathBuf {
    if let Ok(val) = env::var("BRO_CONFIG") {
        let p = PathBuf::from(&val);
        let dir = if p.is_file() {
            p.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            p
        };
        return dir.join("state.toml");
    }
    project_dirs()
        .map(|d| d.config_dir().join("state.toml"))
        .unwrap_or_else(|| PathBuf::from("state.toml"))
}

/// Walk up from `start` to filesystem root; return path of nearest `.bro` file.
pub fn project_store_path() -> Option<PathBuf> {
    project_store_path_from(&env::current_dir().ok()?)
}

pub fn project_store_path_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(".bro");
        if candidate.is_file() {
            return Some(candidate);
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn bro_config_file_used_directly() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("my_aliases.toml");
        fs::write(&file, "").unwrap();
        unsafe { env::set_var("BRO_CONFIG", &file); }
        assert_eq!(global_store_path(), file);
        unsafe { env::remove_var("BRO_CONFIG"); }
    }

    #[test]
    fn bro_config_dir_appends_aliases_toml() {
        let tmp = TempDir::new().unwrap();
        unsafe { env::set_var("BRO_CONFIG", tmp.path()); }
        assert_eq!(global_store_path(), tmp.path().join("aliases.toml"));
        unsafe { env::remove_var("BRO_CONFIG"); }
    }

    #[test]
    fn project_store_walks_up() {
        let tmp = TempDir::new().unwrap();
        let bro_file = tmp.path().join(".bro");
        fs::write(&bro_file, "").unwrap();
        // start from a nested subdir
        let nested = tmp.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        let found = project_store_path_from(&nested);
        assert_eq!(found, Some(bro_file));
    }

    #[test]
    fn project_store_none_when_absent() {
        let tmp = TempDir::new().unwrap();
        let found = project_store_path_from(tmp.path());
        assert!(found.is_none());
    }
}
