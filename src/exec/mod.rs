use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::process;

use anyhow::{bail, Result};

use crate::cli::RunArgs;
use crate::classify::is_stateful;
use crate::resolve::resolve;
use crate::shell::{registry_from_str, InjectionMode};
use crate::stats::UsageState;

pub struct Context {
    pub emit: bool,
    pub shell_name: String,
    pub exec_file: Option<String>,
    pub dry_run: bool,
}

/// Entry point for `bro run [--chain a,b,c | <name> [args...]]`
pub fn run_cmd(args: RunArgs, ctx: &Context) -> Result<()> {
    if let Some(chain) = args.chain {
        let names: Vec<&str> = chain.split(',').map(str::trim).collect();
        run_chain(&names, &[], ctx)
    } else {
        let name = match args.name.as_deref() {
            Some(n) => n,
            None    => bail!("provide an alias name or --chain"),
        };
        run_one(name, &args.args, ctx)
    }
}

/// Entry point for `bro <alias> [args...]` (External dispatch)
pub fn run_external(v: Vec<String>, ctx: &Context) -> Result<()> {
    let name = &v[0];
    let extra = &v[1..];
    run_one(name, extra, ctx)
}

fn run_one(name: &str, extra_args: &[String], ctx: &Context) -> Result<()> {
    let shell = registry_from_str(&ctx.shell_name)?;
    let resolved = resolve(name)?
        .ok_or_else(|| anyhow::anyhow!("alias '{}' not found", name))?;

    let cmd_str = substitute_args(&resolved.alias.cmd, extra_args, shell.as_ref());

    let stateful = match resolved.alias.shell {
        Some(v) => v,
        None    => is_stateful(&cmd_str, shell.kind()),
    };

    // confirm guard — prompt on stderr/tty before emitting or executing
    if !ctx.dry_run && resolved.alias.confirm == Some(true) {
        if !prompt_confirm(name, &cmd_str)? {
            eprintln!("aborted");
            return Ok(());
        }
    }

    emit_or_exec(&cmd_str, stateful, name, ctx, shell.as_ref())?;

    if !ctx.dry_run {
        UsageState::bump(name);
    }
    Ok(())
}

fn run_chain(names: &[&str], extra_args: &[String], ctx: &Context) -> Result<()> {
    let shell = registry_from_str(&ctx.shell_name)?;
    let mut cmds: Vec<String> = Vec::with_capacity(names.len());
    let mut any_stateful = false;

    for &name in names {
        let resolved = resolve(name)?
            .ok_or_else(|| anyhow::anyhow!("alias '{}' not found", name))?;
        let cmd_str = substitute_args(&resolved.alias.cmd, extra_args, shell.as_ref());
        let stateful = match resolved.alias.shell {
            Some(v) => v,
            None    => is_stateful(&cmd_str, shell.kind()),
        };
        if stateful { any_stateful = true; }

        if !ctx.dry_run && resolved.alias.confirm == Some(true) {
            if !prompt_confirm(name, &cmd_str)? {
                eprintln!("aborted");
                return Ok(());
            }
        }

        cmds.push(cmd_str);
    }

    let block = shell.sequence(&cmds);
    emit_or_exec(&block, any_stateful, &names.join(","), ctx, shell.as_ref())?;

    if !ctx.dry_run {
        for &name in names { UsageState::bump(name); }
    }
    Ok(())
}

fn prompt_confirm(name: &str, cmd: &str) -> Result<bool> {
    eprint!("run '{}' ({})? [y/N]: ", name, cmd);
    std::io::stderr().flush()?;
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    let t = input.trim().to_lowercase();
    Ok(t == "y" || t == "yes")
}

/// Either emit code to stdout/file (wrapped) or spawn a child process (unwrapped).
fn emit_or_exec(
    cmd_str: &str,
    stateful: bool,
    name: &str,
    ctx: &Context,
    shell: &dyn crate::shell::Shell,
) -> Result<()> {
    if ctx.dry_run {
        eprintln!("dry-run [{}]: {}", name, cmd_str);
        return Ok(());
    }

    if ctx.emit {
        match shell.injection_mode() {
            InjectionMode::EvalStdout => {
                println!("{}", cmd_str);
            }
            InjectionMode::TempFileCall => {
                let path = ctx.exec_file.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("--exec-file required for cmd.exe mode"))?;
                std::fs::write(path, cmd_str)?;
            }
        }
    } else {
        // Unwrapped fallback
        if stateful {
            eprintln!(
                "warning: '{}' changes shell state (cd, export, source, …). \
                 Run `bro init <shell>` to install the wrapper so state persists.",
                name
            );
        }
        let exit = spawn_child(cmd_str)?;
        if exit != 0 {
            process::exit(exit);
        }
    }
    Ok(())
}

/// Substitute extra args into the command template.
///
/// If the template contains `{}`, `{N}`, or `{name}` placeholders, substitutes them;
/// otherwise falls back to appending all extra args.
pub fn substitute_args(cmd: &str, extra_args: &[String], shell: &dyn crate::shell::Shell) -> String {
    if extra_args.is_empty() {
        return cmd.to_string();
    }
    if !has_placeholders(cmd) {
        let quoted: Vec<String> = extra_args.iter().map(|a| shell.quote(a)).collect();
        return format!("{} {}", cmd, quoted.join(" "));
    }
    let (positional, named) = parse_placeholder_args(extra_args);
    substitute_placeholders(cmd, &positional, &named, shell)
}

fn has_placeholders(cmd: &str) -> bool {
    cmd.find('{').map_or(false, |i| cmd[i + 1..].contains('}'))
}

/// Split extra_args into positional values and `--key value` named pairs.
fn parse_placeholder_args(extra_args: &[String]) -> (Vec<String>, HashMap<String, String>) {
    let mut positional = Vec::new();
    let mut named = HashMap::new();
    let mut i = 0;
    while i < extra_args.len() {
        if let Some(key) = extra_args[i].strip_prefix("--") {
            if i + 1 < extra_args.len() && !extra_args[i + 1].starts_with("--") {
                named.insert(key.to_string(), extra_args[i + 1].clone());
                i += 2;
            } else {
                i += 1;
            }
        } else {
            positional.push(extra_args[i].clone());
            i += 1;
        }
    }
    (positional, named)
}

fn substitute_placeholders(
    cmd: &str,
    positional: &[String],
    named: &HashMap<String, String>,
    shell: &dyn crate::shell::Shell,
) -> String {
    let mut result = String::new();
    let chars: Vec<char> = cmd.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut auto_idx = 0usize;

    while i < n {
        if chars[i] == '{' {
            let start = i + 1;
            if let Some(rel) = chars[start..].iter().position(|&c| c == '}') {
                let key: String = chars[start..start + rel].iter().collect();
                let val = if key.is_empty() {
                    let v = positional.get(auto_idx).cloned().unwrap_or_default();
                    auto_idx += 1;
                    v
                } else if let Ok(n) = key.parse::<usize>() {
                    positional.get(n.saturating_sub(1)).cloned().unwrap_or_default()
                } else {
                    named.get(&key).cloned().unwrap_or_default()
                };
                result.push_str(&shell.quote(&val));
                i = start + rel + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn spawn_child(cmd_str: &str) -> Result<i32> {
    #[cfg(unix)]
    let status = {
        process::Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .status()?
    };

    #[cfg(windows)]
    let status = {
        process::Command::new("cmd")
            .args(["/C", cmd_str])
            .status()?
    };

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::posix::Posix;
    use crate::shell::ShellKind;

    fn bash() -> Posix { Posix { kind: ShellKind::Bash } }

    #[test]
    fn substitute_no_extra_args_unchanged() {
        let sh = bash();
        assert_eq!(substitute_args("git status", &[], &sh), "git status");
    }

    #[test]
    fn substitute_appends_plain_arg() {
        let sh = bash();
        let args = vec!["main".to_string()];
        assert_eq!(substitute_args("git checkout", &args, &sh), "git checkout main");
    }

    #[test]
    fn substitute_quotes_arg_with_spaces() {
        let sh = bash();
        let args = vec!["my branch".to_string()];
        assert_eq!(substitute_args("git checkout", &args, &sh), "git checkout 'my branch'");
    }

    #[test]
    fn substitute_multiple_args() {
        let sh = bash();
        let args = vec!["foo".to_string(), "bar baz".to_string()];
        assert_eq!(substitute_args("echo", &args, &sh), "echo foo 'bar baz'");
    }

    #[test]
    fn placeholder_auto_positional() {
        let sh = bash();
        let args = vec!["manifest.yaml".to_string()];
        assert_eq!(
            substitute_args("kubectl apply -f {}", &args, &sh),
            "kubectl apply -f manifest.yaml"
        );
    }

    #[test]
    fn placeholder_named_arg() {
        let sh = bash();
        let args = vec!["manifest.yaml".to_string(), "--ns".to_string(), "vphs".to_string()];
        assert_eq!(
            substitute_args("kubectl apply -f {} -n {ns}", &args, &sh),
            "kubectl apply -f manifest.yaml -n vphs"
        );
    }

    #[test]
    fn placeholder_numbered_positional() {
        let sh = bash();
        let args = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            substitute_args("echo {2} then {1}", &args, &sh),
            "echo b then a"
        );
    }

    #[test]
    fn no_placeholders_falls_back_to_append() {
        let sh = bash();
        let args = vec!["--oneline".to_string()];
        assert_eq!(
            substitute_args("git log", &args, &sh),
            "git log --oneline"
        );
    }

    #[test]
    fn placeholder_quotes_value_with_spaces() {
        let sh = bash();
        let args = vec!["hello world".to_string()];
        assert_eq!(
            substitute_args("echo {}", &args, &sh),
            "echo 'hello world'"
        );
    }
}
