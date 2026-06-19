use std::process;

use anyhow::{bail, Result};

use crate::cli::RunArgs;
use crate::classify::is_stateful;
use crate::resolve::resolve;
use crate::shell::{registry_from_str, InjectionMode};

pub struct Context {
    pub emit: bool,
    pub shell_name: String,
    pub exec_file: Option<String>,
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

    // Explicit shell field overrides is_stateful detection
    let stateful = match resolved.alias.shell {
        Some(v) => v,
        None    => is_stateful(&cmd_str, shell.kind()),
    };

    emit_or_exec(&cmd_str, stateful, name, ctx, shell.as_ref())
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
        cmds.push(cmd_str);
    }

    let block = shell.sequence(&cmds);
    emit_or_exec(&block, any_stateful, &names.join(","), ctx, shell.as_ref())
}

/// Either emit code to stdout/file (wrapped) or spawn a child process (unwrapped).
fn emit_or_exec(
    cmd_str: &str,
    stateful: bool,
    name: &str,
    ctx: &Context,
    shell: &dyn crate::shell::Shell,
) -> Result<()> {
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

/// v1: append extra args (quoted) to the command string.
/// Placeholder substitution ({} / {name}) hooks in here later (Extension 1).
pub fn substitute_args(cmd: &str, extra_args: &[String], shell: &dyn crate::shell::Shell) -> String {
    if extra_args.is_empty() {
        return cmd.to_string();
    }
    let quoted: Vec<String> = extra_args.iter().map(|a| shell.quote(a)).collect();
    format!("{} {}", cmd, quoted.join(" "))
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
}
