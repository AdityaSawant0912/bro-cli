use anyhow::{bail, Result};

use crate::cli::InfoArgs;
use crate::config::{global_store_path, project_store_path};
use crate::store::Store;

pub fn run(args: InfoArgs) -> Result<()> {
    let project_path = project_store_path();
    let global_path  = global_store_path();

    let project_store = project_path.as_deref().map(Store::load).transpose()?.unwrap_or_default();
    let global_store  = Store::load(&global_path)?;

    if let Some(alias) = project_store.get(&args.name) {
        let path = project_path.as_deref().unwrap();
        eprintln!("name:    {}", args.name);
        eprintln!("source:  project ({})", path.display());
        eprintln!("cmd:     {}", alias.cmd);
        if let Some(ref d) = alias.desc { eprintln!("desc:    {}", d); }
        eprintln!("shell:   {}", fmt_shell(alias.shell));
        if global_store.get(&args.name).is_some() {
            eprintln!("warning: shadows a global alias with the same name");
        }
        return Ok(());
    }

    if let Some(alias) = global_store.get(&args.name) {
        eprintln!("name:    {}", args.name);
        eprintln!("source:  global ({})", global_path.display());
        eprintln!("cmd:     {}", alias.cmd);
        if let Some(ref d) = alias.desc { eprintln!("desc:    {}", d); }
        eprintln!("shell:   {}", fmt_shell(alias.shell));
        return Ok(());
    }

    bail!("alias '{}' not found", args.name);
}

fn fmt_shell(s: Option<bool>) -> &'static str {
    match s {
        Some(true)  => "true (forced)",
        Some(false) => "false (forced)",
        None        => "auto-detect",
    }
}
