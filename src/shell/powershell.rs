use std::path::Path;

use anyhow::Result;

use super::{InjectionMode, Shell, ShellKind};

pub struct Ps;

impl Shell for Ps {
    fn kind(&self) -> ShellKind {
        ShellKind::PowerShell
    }

    fn injection_mode(&self) -> InjectionMode {
        InjectionMode::EvalStdout
    }

    fn quote(&self, arg: &str) -> String {
        // Single-quote with '' escaping for PowerShell
        format!("'{}'", arg.replace('\'', "''"))
    }

    fn sequence(&self, cmds: &[String]) -> String {
        cmds.join(";\n")
    }

    fn init_script(&self, bin: &Path) -> Result<String> {
        let bin_str = bin.display();
        Ok(format!(
            r#"# bro wrapper — add to $PROFILE:
# Invoke-Expression (& bro init powershell | Out-String)
function bro {{
  $code = & '{bin}' --emit --shell-name powershell run @args
  if ($LASTEXITCODE -ne 0) {{ return }}
  Invoke-Expression ($code -join "`n")
}}
"#,
            bin = bin_str,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_plain() {
        assert_eq!(Ps.quote("hello"), "'hello'");
    }

    #[test]
    fn quote_with_single_quote() {
        assert_eq!(Ps.quote("it's"), "'it''s'");
    }

    #[test]
    fn sequence_joins_with_semicolon_newline() {
        let cmds = vec!["Set-Location C:\\foo".to_string(), "ls".to_string()];
        assert_eq!(Ps.sequence(&cmds), "Set-Location C:\\foo;\nls");
    }

    #[test]
    fn init_script_contains_invoke_expression() {
        let script = Ps.init_script(Path::new("C:\\bin\\bro.exe")).unwrap();
        assert!(script.contains("Invoke-Expression"));
        assert!(script.contains("--emit"));
        assert!(script.contains("--shell-name powershell"));
    }
}
