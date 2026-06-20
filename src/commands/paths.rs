use anyhow::Result;

use crate::config::{global_store_path, project_store_path};

pub fn run() -> Result<()> {
    let global = global_store_path();
    eprintln!("global store:  {}", global.display());

    match project_store_path() {
        Some(p) => eprintln!("project store: {}", p.display()),
        None    => eprintln!("project store: none (no .bro found in this directory tree)"),
    }

    Ok(())
}
