use anyhow::Result;

use crate::cli::UpdateArgs;
use crate::store::{model::Alias, Store};
use super::{expand_cmd, target_path};

pub fn run(args: UpdateArgs) -> Result<()> {
    let path = target_path(args.local)?;
    let mut store = Store::load(&path)?;

    let existing = store.get(&args.name)
        .ok_or_else(|| anyhow::anyhow!("alias '{}' not found in {}", args.name, path.display()))?
        .clone();

    // Merge: explicit args override, unspecified fields keep existing values.
    let cmd = if args.value.is_some() || args.py.is_some() || args.js.is_some() {
        expand_cmd(args.value.as_deref(), args.py.as_deref(), args.js.as_deref())?
    } else {
        existing.cmd
    };
    let shell = match (args.shell, args.no_shell) {
        (true, _)  => Some(true),
        (_, true)  => Some(false),
        _          => existing.shell,
    };
    let desc    = if args.desc.is_some()  { args.desc }  else { existing.desc };
    let tags    = if !args.tag.is_empty() { args.tag }   else { existing.tags };
    let confirm = match (args.confirm, args.no_confirm) {
        (true, _)  => Some(true),
        (_, true)  => Some(false),
        _          => existing.confirm,
    };

    let alias = Alias { cmd, shell, desc, tags, confirm };
    store.insert(&args.name, alias);
    store.save(&path)?;

    eprintln!("updated '{}'", args.name);
    Ok(())
}
