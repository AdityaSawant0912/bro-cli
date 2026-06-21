use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;
use dialoguer::FuzzySelect;
use dialoguer::theme::ColorfulTheme;

use crate::config::{global_store_path, project_store_path};
use crate::exec::{self, Context};
use crate::store::Store;

pub fn run(ctx: &Context) -> Result<()> {
    let aliases = load_aliases()?;
    if aliases.is_empty() {
        eprintln!("no aliases defined");
        return Ok(());
    }

    let names: Vec<String> = aliases.keys().cloned().collect(); // BTreeMap = sorted

    let selected = if fzf_available() {
        pick_with_fzf(&names, &aliases)?
    } else {
        pick_with_dialoguer(&names, &aliases)?
    };

    if let Some(name) = selected {
        exec::run_external(vec![name], ctx)?;
    }
    Ok(())
}

// Project aliases shadow global — load global first then override with project.
fn load_aliases() -> Result<BTreeMap<String, String>> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    if let Ok(s) = Store::load(&global_store_path()) {
        for (k, v) in s.aliases { map.insert(k, v.cmd); }
    }
    if let Some(p) = project_store_path() {
        if let Ok(s) = Store::load(&p) {
            for (k, v) in s.aliases { map.insert(k, v.cmd); }
        }
    }
    Ok(map)
}

fn fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Pipe name+cmd pairs to fzf. fzf opens /dev/tty for its UI; selection comes
/// back on fzf's stdout which we capture, so binary's stdout stays clean for
/// the wrapper to eval.
fn pick_with_fzf(names: &[String], aliases: &BTreeMap<String, String>) -> Result<Option<String>> {
    let input = names
        .iter()
        .map(|n| {
            let cmd = aliases.get(n).map(String::as_str).unwrap_or("");
            format!("{}\t{}", n, cmd)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .args(["--delimiter", "\t", "--with-nth", "1,2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None); // user cancelled (Esc / Ctrl-C)
    }

    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let name = line.split('\t').next().unwrap_or("").trim().to_string();
    if name.is_empty() { Ok(None) } else { Ok(Some(name)) }
}

fn pick_with_dialoguer(names: &[String], aliases: &BTreeMap<String, String>) -> Result<Option<String>> {
    let w = names.iter().map(|n| n.len()).max().unwrap_or(4);

    // Build display items: "name    cmd" — dialoguer fuzzy-searches over these.
    let items: Vec<String> = names
        .iter()
        .map(|n| {
            let cmd = aliases.get(n).map(String::as_str).unwrap_or("");
            format!("{:<w$}  {}", n, cmd, w = w)
        })
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("pick alias")
        .items(&items)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i: usize| names[i].clone()))
}
