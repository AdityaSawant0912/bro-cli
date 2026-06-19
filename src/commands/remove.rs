use anyhow::{bail, Result};

use crate::cli::RemoveArgs;
use crate::store::Store;
use super::target_path;

pub fn run(args: RemoveArgs) -> Result<()> {
    let path = target_path(args.local)?;
    let mut store = Store::load(&path)?;

    if !store.remove(&args.name) {
        bail!("alias '{}' not found in {}", args.name, path.display());
    }

    store.save(&path)?;
    eprintln!("removed '{}'", args.name);
    Ok(())
}
