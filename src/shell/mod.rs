pub mod cmd;
pub mod posix;
pub mod powershell;

use std::path::Path;

use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

impl ShellKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash"                => Some(ShellKind::Bash),
            "zsh"                 => Some(ShellKind::Zsh),
            "fish"                => Some(ShellKind::Fish),
            "powershell" | "pwsh" => Some(ShellKind::PowerShell),
            "cmd"                 => Some(ShellKind::Cmd),
            _                     => None,
        }
    }

    pub fn is_posix(self) -> bool {
        matches!(self, ShellKind::Bash | ShellKind::Zsh | ShellKind::Fish)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionMode {
    /// Binary prints shell code to stdout; wrapper evals it.
    EvalStdout,
    /// Binary writes shell code to a temp file; wrapper calls it (cmd.exe).
    TempFileCall,
}

pub trait Shell {
    fn kind(&self) -> ShellKind;
    fn injection_mode(&self) -> InjectionMode;
    /// Quote a single argument for safe inclusion in a shell command.
    fn quote(&self, arg: &str) -> String;
    /// Join multiple commands preserving execution order.
    fn sequence(&self, cmds: &[String]) -> String;
    /// Emit the wrapper function the user sources once to install bro.
    fn init_script(&self, bin: &Path) -> Result<String>;
}

pub fn registry(kind: ShellKind) -> Box<dyn Shell> {
    match kind {
        ShellKind::Bash       => Box::new(posix::Posix { kind: ShellKind::Bash }),
        ShellKind::Zsh        => Box::new(posix::Posix { kind: ShellKind::Zsh }),
        ShellKind::Fish       => Box::new(posix::Posix { kind: ShellKind::Fish }),
        ShellKind::PowerShell => Box::new(powershell::Ps),
        ShellKind::Cmd        => Box::new(cmd::Cmd),
    }
}

pub fn registry_from_str(s: &str) -> Result<Box<dyn Shell>> {
    match ShellKind::from_str(s) {
        Some(k) => Ok(registry(k)),
        None    => bail!("unknown shell '{}' (supported: bash, zsh, fish, powershell, cmd)", s),
    }
}
