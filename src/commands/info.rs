use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};

use crate::cli::InfoArgs;
use crate::config::{global_store_path, project_store_path};
use crate::stats::UsageState;
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
        if !alias.tags.is_empty() { eprintln!("tags:    {}", alias.tags.join(", ")); }
        if alias.confirm == Some(true) { eprintln!("confirm: yes"); }
        print_usage(&args.name);
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
        if !alias.tags.is_empty() { eprintln!("tags:    {}", alias.tags.join(", ")); }
        if alias.confirm == Some(true) { eprintln!("confirm: yes"); }
        print_usage(&args.name);
        return Ok(());
    }

    bail!("alias '{}' not found", args.name);
}

fn print_usage(name: &str) {
    let state = UsageState::load();
    if let Some(entry) = state.usage.get(name) {
        if entry.count > 0 {
            eprintln!("runs:    {}", entry.count);
            eprintln!("last:    {}", fmt_elapsed(entry.last_used));
        }
    }
}

fn fmt_elapsed(ts: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if ts == 0 || now < ts { return "unknown".to_string(); }
    let diff = now - ts;
    if diff < 60    { return "just now".to_string(); }
    if diff < 3600  { return format!("{} min ago", diff / 60); }
    if diff < 86400 { return format!("{} hr ago", diff / 3600); }
    format!("{} days ago", diff / 86400)
}

fn fmt_shell(s: Option<bool>) -> &'static str {
    match s {
        Some(true)  => "true (forced)",
        Some(false) => "false (forced)",
        None        => "auto-detect",
    }
}
