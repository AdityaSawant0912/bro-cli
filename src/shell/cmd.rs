use std::path::Path;

use anyhow::{bail, Result};

use super::{InjectionMode, Shell, ShellKind};

pub struct Cmd;

impl Shell for Cmd {
    fn kind(&self) -> ShellKind {
        ShellKind::Cmd
    }

    fn injection_mode(&self) -> InjectionMode {
        InjectionMode::TempFileCall
    }

    fn quote(&self, arg: &str) -> String {
        // Basic cmd.exe quoting: wrap in double-quotes, escape internal double-quotes
        if arg.contains(' ') || arg.contains('"') || arg.contains('&') {
            format!("\"{}\"", arg.replace('"', "\"\""))
        } else {
            arg.to_string()
        }
    }

    fn sequence(&self, cmds: &[String]) -> String {
        cmds.join(" &\n")
    }

    fn init_script(&self, _bin: &Path) -> Result<String> {
        bail!("cmd.exe support is not yet implemented — coming soon");
    }
}
