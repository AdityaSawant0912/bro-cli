use anyhow::Result;

use crate::cli::ListArgs;
use crate::config::{global_store_path, project_store_path};
use crate::stats::UsageState;
use crate::store::Store;

struct Row {
    name:   String,
    source: String,
    shell:  String,
    cmd:    String,
    desc:   String,
    tags:   String,
    count:  u64,
}

pub fn run(args: ListArgs) -> Result<()> {
    let project_path = project_store_path();
    let global_path  = global_store_path();

    let project_store = project_path.as_deref().map(Store::load).transpose()?.unwrap_or_default();
    let global_store  = if args.local { Store::default() } else { Store::load(&global_path)? };

    let usage = if args.by_usage { Some(UsageState::load()) } else { None };

    let mut rows: Vec<Row> = Vec::new();

    if !args.global {
        for (name, alias) in &project_store.aliases {
            if let Some(ref tag) = args.tag {
                if !alias.tags.iter().any(|t| t == tag) { continue; }
            }
            let shadowed = global_store.get(name).is_some();
            rows.push(Row {
                name:   name.clone(),
                source: if shadowed { "project (shadows global)".into() } else { "project".into() },
                shell:  fmt_shell(alias.shell),
                cmd:    alias.cmd.clone(),
                desc:   alias.desc.clone().unwrap_or_default(),
                tags:   fmt_tags(&alias.tags),
                count:  usage.as_ref().and_then(|u| u.usage.get(name)).map(|e| e.count).unwrap_or(0),
            });
        }
    }

    if !args.local {
        for (name, alias) in &global_store.aliases {
            if let Some(ref tag) = args.tag {
                if !alias.tags.iter().any(|t| t == tag) { continue; }
            }
            let shadowed = project_store.get(name).is_some();
            rows.push(Row {
                name:   name.clone(),
                source: if shadowed { "global (shadowed)".into() } else { "global".into() },
                shell:  fmt_shell(alias.shell),
                cmd:    alias.cmd.clone(),
                desc:   alias.desc.clone().unwrap_or_default(),
                tags:   fmt_tags(&alias.tags),
                count:  usage.as_ref().and_then(|u| u.usage.get(name)).map(|e| e.count).unwrap_or(0),
            });
        }
    }

    if rows.is_empty() {
        eprintln!("no aliases found");
        return Ok(());
    }

    if args.by_usage {
        rows.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    } else {
        rows.sort_by(|a, b| a.name.cmp(&b.name));
    }

    let any_tags  = rows.iter().any(|r| !r.tags.is_empty());
    let any_usage = args.by_usage;

    let w_name   = rows.iter().map(|r| r.name.len()).max().unwrap_or(4).max(4);
    let w_source = rows.iter().map(|r| r.source.len()).max().unwrap_or(6).max(6);
    let w_shell  = 5;
    let w_tags   = if any_tags  { rows.iter().map(|r| r.tags.len()).max().unwrap_or(4).max(4) } else { 0 };

    // Header
    let mut header = format!(
        "{:<w_name$}  {:<w_source$}  {:<w_shell$}",
        "NAME", "SOURCE", "SHELL",
        w_name = w_name, w_source = w_source, w_shell = w_shell
    );
    if any_tags  { header.push_str(&format!("  {:<w_tags$}", "TAGS", w_tags = w_tags)); }
    if any_usage { header.push_str("  RUNS"); }
    header.push_str("  COMMAND");
    eprintln!("{}", header);
    eprintln!("{}", "-".repeat(header.len() + 10));

    for r in &rows {
        let mut line = format!(
            "{:<w_name$}  {:<w_source$}  {:<w_shell$}",
            r.name, r.source, r.shell,
            w_name = w_name, w_source = w_source, w_shell = w_shell
        );
        if any_tags  { line.push_str(&format!("  {:<w_tags$}", r.tags, w_tags = w_tags)); }
        if any_usage { line.push_str(&format!("  {:>4}", r.count)); }
        let cmd_col = if r.desc.is_empty() { r.cmd.clone() } else { format!("{}  # {}", r.cmd, r.desc) };
        line.push_str(&format!("  {}", cmd_col));
        eprintln!("{}", line);
    }

    Ok(())
}

fn fmt_shell(s: Option<bool>) -> String {
    match s {
        Some(true)  => "shell".into(),
        Some(false) => "pure".into(),
        None        => "auto".into(),
    }
}

fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() { String::new() } else { tags.join(",") }
}
