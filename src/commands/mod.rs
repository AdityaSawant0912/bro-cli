pub mod add;
pub mod completions;
pub mod edit;
pub mod info;
pub mod init;
pub mod list;
pub mod paths;
pub mod pick;
pub mod remove;
pub mod search;
pub mod update;

use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::config::{global_store_path, project_store_path};
use crate::store::Store;

/// Resolve target store path. If `local`, requires a project `.bro` to exist (or creates one).
pub(crate) fn target_path(local: bool) -> Result<PathBuf> {
    if local {
        match project_store_path() {
            Some(p) => Ok(p),
            None => {
                // create a minimal .bro in CWD
                let cwd = std::env::current_dir()?;
                let p = cwd.join(".bro");
                let empty = Store::default();
                empty.save(&p)?;
                Ok(p)
            }
        }
    } else {
        Ok(global_store_path())
    }
}

/// Expand --py / --js shorthand into a full command string.
pub(crate) fn expand_cmd(value: Option<&str>, py: Option<&str>, js: Option<&str>) -> Result<String> {
    match (value, py, js) {
        (Some(v), None, None) => Ok(v.to_string()),
        (None, Some(p), None) => Ok(format!("python {}", p)),
        (None, None, Some(j)) => Ok(format!("node {}", j)),
        (None, None, None)    => bail!("provide a command value, --py, or --js"),
        _ => bail!("conflicting flags: provide only one of value / --py / --js"),
    }
}
