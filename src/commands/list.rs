use anyhow::Result;

use crate::cli::ListArgs;
use crate::config::{global_store_path, project_store_path};
use crate::store::Store;

pub fn run(args: ListArgs) -> Result<()> {
    let project_path = project_store_path();
    let global_path  = global_store_path();

    let project_store = project_path.as_deref().map(Store::load).transpose()?.unwrap_or_default();
    let global_store  = if args.local { Store::default() } else { Store::load(&global_path)? };

    // Build rows: (name, source_label, shell_flag, cmd, desc, shadowed)
    let mut rows: Vec<(String, &str, String, String, String, bool)> = Vec::new();

    if !args.global {
        for (name, alias) in &project_store.aliases {
            let shadowed = global_store.get(name).is_some();
            let shell_flag = match alias.shell {
                Some(true)  => "shell",
                Some(false) => "pure",
                None        => "auto",
            };
            rows.push((
                name.clone(),
                "project",
                shell_flag.to_string(),
                alias.cmd.clone(),
                alias.desc.clone().unwrap_or_default(),
                shadowed,
            ));
        }
    }

    if !args.local {
        for (name, alias) in &global_store.aliases {
            let shadowed_by_project = project_store.get(name).is_some();
            let shell_flag = match alias.shell {
                Some(true)  => "shell",
                Some(false) => "pure",
                None        => "auto",
            };
            rows.push((
                name.clone(),
                if shadowed_by_project { "global (shadowed)" } else { "global" },
                shell_flag.to_string(),
                alias.cmd.clone(),
                alias.desc.clone().unwrap_or_default(),
                false,
            ));
        }
    }

    if rows.is_empty() {
        eprintln!("no aliases found");
        return Ok(());
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));

    let w_name   = rows.iter().map(|r| r.0.len()).max().unwrap_or(4).max(4);
    let w_source = rows.iter().map(|r| r.1.len()).max().unwrap_or(6).max(6);
    let w_shell  = 5;

    eprintln!(
        "{:<w_name$}  {:<w_source$}  {:<w_shell$}  {}",
        "NAME", "SOURCE", "SHELL", "COMMAND",
        w_name = w_name, w_source = w_source, w_shell = w_shell
    );
    eprintln!("{}", "-".repeat(w_name + w_source + w_shell + 30));

    for (name, source, shell, cmd, desc, _) in &rows {
        let cmd_col = if desc.is_empty() {
            cmd.clone()
        } else {
            format!("{}  # {}", cmd, desc)
        };
        eprintln!(
            "{:<w_name$}  {:<w_source$}  {:<w_shell$}  {}",
            name, source, shell, cmd_col,
            w_name = w_name, w_source = w_source, w_shell = w_shell
        );
    }

    Ok(())
}
