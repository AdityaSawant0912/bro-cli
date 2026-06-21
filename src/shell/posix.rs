use std::path::Path;

use anyhow::Result;

use super::{InjectionMode, Shell, ShellKind};

pub struct Posix {
    pub kind: ShellKind,
}

impl Shell for Posix {
    fn kind(&self) -> ShellKind {
        self.kind
    }

    fn injection_mode(&self) -> InjectionMode {
        InjectionMode::EvalStdout
    }

    fn quote(&self, arg: &str) -> String {
        shlex::try_quote(arg)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| format!("'{}'", arg.replace('\'', "'\\''")))
    }

    fn sequence(&self, cmds: &[String]) -> String {
        cmds.join("\n")
    }

    fn init_script(&self, bin: &Path) -> Result<String> {
        let bin_str = bin.display();
        let shell_name = match self.kind {
            ShellKind::Fish => "fish",
            ShellKind::Zsh  => "zsh",
            _               => "bash",
        };

        if self.kind == ShellKind::Fish {
            return Ok(format!(
                r#"# bro wrapper — add to ~/.config/fish/config.fish:
# bro init fish | source
function bro
    set mgmt add update set remove rm list ls info search find edit init paths run completions help
    if test (count $argv) -eq 0; or contains -- $argv[1] "-f" "pick"
        set code ('{bin}' --emit --shell-name fish pick)
        or return $status
        if set -q code[1]; eval $code; end
    else if contains -- $argv[1] $mgmt; or string match -q -- '-*' $argv[1]
        '{bin}' $argv
    else
        set code ('{bin}' --emit --shell-name fish run $argv)
        or return $status
        if set -q code[1]; eval $code; end
    end
end
"#,
                bin = bin_str
            ));
        }

        Ok(format!(
            r#"# bro wrapper — add to ~/.bashrc or ~/.zshrc:
# eval "$(bro init {shell})"
bro() {{
  local mgmt="add update set remove rm list ls info search find edit init paths run completions help"
  if [[ $# -eq 0 ]] || [[ "${{1:-}}" == "-f" ]] || [[ "${{1:-}}" == "pick" ]]; then
    local out
    out="$('{bin}' --emit --shell-name {shell} pick)" || return $?
    if [[ -n "$out" ]]; then eval "$out"; fi
  elif echo "$mgmt" | grep -qw "${{1:-}}" || [[ "${{1:-}}" == -* ]]; then
    '{bin}' "$@"
  else
    local out
    out="$('{bin}' --emit --shell-name {shell} run "$@")" || return $?
    eval "$out"
  fi
}}
"#,
            shell = shell_name,
            bin   = bin_str,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_plain() {
        let sh = Posix { kind: ShellKind::Bash };
        assert_eq!(sh.quote("hello"), "hello");
    }

    #[test]
    fn quote_with_spaces() {
        let sh = Posix { kind: ShellKind::Bash };
        assert_eq!(sh.quote("hello world"), "'hello world'");
    }

    #[test]
    fn sequence_joins_with_newline() {
        let sh = Posix { kind: ShellKind::Bash };
        let cmds = vec!["cd foo".to_string(), "ls".to_string()];
        assert_eq!(sh.sequence(&cmds), "cd foo\nls");
    }

    #[test]
    fn init_script_contains_eval() {
        let sh = Posix { kind: ShellKind::Bash };
        let script = sh.init_script(Path::new("/usr/local/bin/bro")).unwrap();
        assert!(script.contains("eval"));
        assert!(script.contains("--emit"));
        assert!(script.contains("--shell-name bash"));
    }
}
