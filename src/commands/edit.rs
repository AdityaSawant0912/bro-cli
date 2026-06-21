use std::env;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Result};

use crate::cli::EditArgs;
use crate::config::{global_store_path, project_store_path};
use crate::resolve::{resolve, Source};
use crate::store::Store;

pub fn run(args: EditArgs) -> Result<()> {
    let path = resolve_edit_path(&args)?;

    // Create the file if it doesn't exist yet so the editor can open it.
    if !path.exists() {
        Store::default().save(&path)?;
    }

    let line_num = args.name.as_deref().and_then(|n| find_alias_line(&path, n));

    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| default_editor());

    loop {
        let status = open_editor(&editor, &path, line_num)?;
        if !status.success() {
            bail!("editor exited non-zero ({})", status.code().unwrap_or(-1));
        }
        match Store::load(&path) {
            Ok(_) => break,
            Err(e) => {
                eprintln!("error: invalid TOML after edit — {}", e);
                eprint!("reopen editor to fix? [Y/n]: ");
                std::io::stderr().flush().ok();
                let mut input = String::new();
                match std::io::stdin().lock().read_line(&mut input) {
                    Ok(_) if input.trim().eq_ignore_ascii_case("n") => {
                        bail!("aborted: store has parse errors");
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn resolve_edit_path(args: &EditArgs) -> Result<PathBuf> {
    if let Some(ref name) = args.name {
        match resolve(name)? {
            Some(r) => match r.source {
                Source::Project(p) => return Ok(p),
                Source::Global => return Ok(global_store_path()),
            },
            None => bail!("alias '{}' not found", name),
        }
    }
    if args.local {
        return match project_store_path() {
            Some(p) => Ok(p),
            None => bail!("no .bro project file found in current directory or ancestors"),
        };
    }
    Ok(global_store_path())
}

fn find_alias_line(path: &Path, name: &str) -> Option<usize> {
    let text = std::fs::read_to_string(path).ok()?;
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(name) {
            let after = trimmed[name.len()..].trim_start();
            if after.starts_with('=') {
                return Some(i + 1);
            }
        }
    }
    None
}

fn open_editor(editor: &str, path: &Path, line: Option<usize>) -> Result<std::process::ExitStatus> {
    let editor_lower = editor.to_lowercase();

    if let Some(n) = line {
        if editor_lower.contains("code") {
            // VS Code: `code --goto file:line`
            return Ok(Command::new(editor)
                .arg("--goto")
                .arg(format!("{}:{}", path.display(), n))
                .status()?);
        }
        if editor_lower.contains("vim")
            || editor_lower.contains("nvim")
            || editor_lower.contains("nano")
            || editor_lower.contains("emacs")
        {
            return Ok(Command::new(editor)
                .arg(format!("+{}", n))
                .arg(path)
                .status()?);
        }
    }

    Ok(Command::new(editor).arg(path).status()?)
}

fn default_editor() -> String {
    if cfg!(windows) {
        "notepad".to_string()
    } else {
        "vi".to_string()
    }
}
