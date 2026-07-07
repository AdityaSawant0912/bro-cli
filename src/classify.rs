use crate::shell::ShellKind;

/// Returns true if `cmd` contains any shell-stateful operation for the given shell.
/// Explicit `shell` field on an alias overrides this entirely — call site handles that.
pub fn is_stateful(cmd: &str, shell: ShellKind) -> bool {
    segments(cmd).into_iter().any(|seg| segment_is_stateful(&seg, shell))
}

/// Split a command string into pipeline/list segments.
/// Splits on &&, ||, ;, |, \n. Simple string split — good enough for v1 classification.
fn segments(cmd: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = cmd.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '&' if chars.peek() == Some(&'&') => {
                chars.next();
                result.push(current.trim().to_string());
                current = String::new();
            }
            '|' if chars.peek() == Some(&'|') => {
                chars.next();
                result.push(current.trim().to_string());
                current = String::new();
            }
            ';' | '|' | '\n' => {
                result.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result.into_iter().filter(|s| !s.is_empty()).collect()
}

fn segment_is_stateful(seg: &str, shell: ShellKind) -> bool {
    // PowerShell: env/global variable assignment prefix
    if shell == ShellKind::PowerShell {
        let low = seg.trim_start().to_lowercase();
        if low.starts_with("$env:") || low.starts_with("$global:") {
            return true;
        }
    }

    // Multi-word specials checked before tokenizing
    if is_multiword_stateful(seg) {
        return true;
    }

    // Tokenize and check first token
    let tokens = match shlex::split(seg) {
        Some(t) => t,
        None    => return false, // malformed quoting → assume pure
    };

    let first = match tokens.first() {
        Some(t) => t.as_str(),
        None => return false,
    };

    if shell.is_posix() {
        POSIX_STATEFUL.contains(&first)
    } else if shell == ShellKind::PowerShell {
        POWERSHELL_STATEFUL.iter().any(|&s| s.eq_ignore_ascii_case(first))
    } else {
        // cmd.exe — treat same as POSIX for now
        CMD_STATEFUL.contains(&first)
    }
}

fn is_multiword_stateful(seg: &str) -> bool {
    let low = seg.trim_start().to_lowercase();
    MULTIWORD_STATEFUL.iter().any(|&prefix| low.starts_with(prefix))
}

const POSIX_STATEFUL: &[&str] = &[
    "cd", "pushd", "popd",
    "export", "set", "unset",
    "source", ".",
    "alias",
    "activate", "deactivate",
];

const POWERSHELL_STATEFUL: &[&str] = &[
    "cd", "set-location", "sl",
    "pushd", "push-location",
    "popd", "pop-location",
    ".", // dot-sourcing
    "set-variable", "sv",
    "new-variable", "nv",
    "remove-variable", "rv",
];

const CMD_STATEFUL: &[&str] = &[
    "cd", "chdir", "pushd", "popd", "set",
];

const MULTIWORD_STATEFUL: &[&str] = &[
    "conda activate",
    "conda deactivate",
    "nvm use",
    "nvm install",
    "pyenv shell",
    "rbenv shell",
    "asdf shell",
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- POSIX ---

    #[test]
    fn posix_cd_is_stateful() {
        assert!(is_stateful("cd ~/projects", ShellKind::Bash));
    }

    #[test]
    fn posix_source_is_stateful() {
        assert!(is_stateful("source venv/bin/activate", ShellKind::Bash));
    }

    #[test]
    fn posix_dot_source_is_stateful() {
        assert!(is_stateful(". venv/bin/activate", ShellKind::Bash));
    }

    #[test]
    fn posix_export_is_stateful() {
        assert!(is_stateful("export FOO=bar", ShellKind::Bash));
    }

    #[test]
    fn posix_chain_with_cd_is_stateful() {
        assert!(is_stateful("git fetch && cd ../other", ShellKind::Bash));
    }

    #[test]
    fn posix_pure_command_is_not_stateful() {
        assert!(!is_stateful("git status", ShellKind::Bash));
    }

    #[test]
    fn posix_pipe_pure_is_not_stateful() {
        assert!(!is_stateful("ls | grep foo", ShellKind::Bash));
    }

    // --- PowerShell ---

    #[test]
    fn ps_set_location_is_stateful() {
        assert!(is_stateful("Set-Location C:\\projects", ShellKind::PowerShell));
    }

    #[test]
    fn ps_cd_alias_is_stateful() {
        assert!(is_stateful("cd C:\\projects", ShellKind::PowerShell));
    }

    #[test]
    fn ps_env_assignment_is_stateful() {
        assert!(is_stateful("$env:FOO = 'bar'", ShellKind::PowerShell));
    }

    #[test]
    fn ps_global_assignment_is_stateful() {
        assert!(is_stateful("$global:x = 1", ShellKind::PowerShell));
    }

    #[test]
    fn ps_pure_is_not_stateful() {
        assert!(!is_stateful("git status", ShellKind::PowerShell));
    }

    // --- multi-word specials ---

    #[test]
    fn conda_activate_is_stateful() {
        assert!(is_stateful("conda activate myenv", ShellKind::Bash));
    }

    #[test]
    fn nvm_use_is_stateful() {
        assert!(is_stateful("nvm use 20", ShellKind::Bash));
    }

    #[test]
    fn pyenv_shell_is_stateful() {
        assert!(is_stateful("pyenv shell 3.12.0", ShellKind::Zsh));
    }
}
