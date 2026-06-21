use anyhow::{bail, Result};

use crate::cli::AddArgs;
use crate::store::{model::Alias, Store};
use super::{expand_cmd, target_path};

pub fn run(args: AddArgs) -> Result<()> {
    let path = target_path(args.local)?;
    let mut store = Store::load(&path)?;

    if store.get(&args.name).is_some() {
        bail!("alias '{}' already exists (use update to change it)", args.name);
    }

    let cmd = expand_cmd(args.value.as_deref(), args.py.as_deref(), args.js.as_deref())?;
    let shell = match (args.shell, args.no_shell) {
        (true, _)  => Some(true),
        (_, true)  => Some(false),
        _          => None,
    };
    let confirm = match (args.confirm, args.no_confirm) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _         => None,
    };
    let alias = Alias { cmd, shell, desc: args.desc, tags: args.tag, confirm };
    store.insert(&args.name, alias);
    store.save(&path)?;

    eprintln!("added '{}'", args.name);
    Ok(())
}
