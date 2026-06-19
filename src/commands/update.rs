use anyhow::{bail, Result};

use crate::cli::UpdateArgs;
use crate::store::{model::Alias, Store};
use super::{expand_cmd, target_path};

pub fn run(args: UpdateArgs) -> Result<()> {
    let path = target_path(args.local)?;
    let mut store = Store::load(&path)?;

    if store.get(&args.name).is_none() {
        bail!("alias '{}' not found in {}", args.name, path.display());
    }

    let cmd = expand_cmd(args.value.as_deref(), args.py.as_deref(), args.js.as_deref())?;
    let shell = match (args.shell, args.no_shell) {
        (true, _)  => Some(true),
        (_, true)  => Some(false),
        _          => None,
    };
    let alias = Alias { cmd, shell, desc: args.desc };
    store.insert(&args.name, alias);
    store.save(&path)?;

    eprintln!("updated '{}'", args.name);
    Ok(())
}
