use std::collections::BTreeSet;

use anyhow::{bail, Result};

use crate::cli::CompletionsArgs;
use crate::config::{global_store_path, project_store_path};
use crate::store::Store;

/// Called by `bro --complete <prefix>` — prints matching alias names, one per line.
pub fn print_completions(prefix: &str) -> Result<()> {
    let mut names: BTreeSet<String> = BTreeSet::new();

    if let Ok(s) = Store::load(&global_store_path()) {
        names.extend(s.aliases.into_keys());
    }
    if let Some(p) = project_store_path() {
        if let Ok(s) = Store::load(&p) {
            names.extend(s.aliases.into_keys());
        }
    }

    for name in names {
        if name.starts_with(prefix) {
            println!("{}", name);
        }
    }
    Ok(())
}

/// Called by `bro completions <shell>` — emits a tab-completion script.
pub fn run(args: CompletionsArgs) -> Result<()> {
    let script = match args.shell.to_lowercase().as_str() {
        "bash"       => bash_script(),
        "zsh"        => zsh_script(),
        "fish"       => fish_script(),
        "powershell" | "pwsh" => powershell_script(),
        other => bail!(
            "unsupported shell '{}' (supported: bash, zsh, fish, powershell)",
            other
        ),
    };
    print!("{}", script);
    Ok(())
}

fn bash_script() -> String {
    r#"# bro bash completion — source this or add to ~/.bashrc:
# source <(bro completions bash)
_bro_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local cmd="${COMP_WORDS[1]}"
    local subcmds="add update set remove rm list ls info search edit init paths run completions"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        local aliases
        aliases=$(bro --complete "$cur" 2>/dev/null)
        COMPREPLY=($(compgen -W "$subcmds $aliases" -- "$cur"))
    elif [[ $cmd == "run" && ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=($(compgen -W "$(bro --complete "$cur" 2>/dev/null)" -- "$cur"))
    elif [[ $cmd == "info" || $cmd == "update" || $cmd == "set" || \
            $cmd == "remove" || $cmd == "rm" || $cmd == "edit" ]] && \
         [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=($(compgen -W "$(bro --complete "$cur" 2>/dev/null)" -- "$cur"))
    elif [[ ($cmd == "init" || $cmd == "completions") && ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"))
    fi
    return 0
}
complete -F _bro_complete bro
"#.to_string()
}

fn zsh_script() -> String {
    r#"# bro zsh completion — add to a $fpath directory or source directly:
# source <(bro completions zsh)
_bro() {
    local state
    local -a subcmds aliases

    subcmds=(
        'add:Add an alias'
        'update:Update an alias'
        'set:Update an alias'
        'remove:Remove an alias'
        'rm:Remove an alias'
        'list:List all aliases'
        'ls:List all aliases'
        'info:Show alias details'
        'search:Search alias names and commands'
        'edit:Open store in $EDITOR'
        'init:Emit shell wrapper'
        'paths:Show store locations'
        'run:Run an alias explicitly'
        'completions:Emit tab-completion script'
    )

    _arguments '1: :->cmd' '*: :->args'

    case $state in
        cmd)
            aliases=(${(f)"$(bro --complete "$PREFIX" 2>/dev/null)"})
            _describe 'subcommand' subcmds
            [[ ${#aliases} -gt 0 ]] && _describe 'alias' aliases
            ;;
        args)
            case ${words[2]} in
                run|info|update|set|remove|rm|edit)
                    aliases=(${(f)"$(bro --complete "$PREFIX" 2>/dev/null)"})
                    [[ ${#aliases} -gt 0 ]] && _describe 'alias' aliases
                    ;;
                init|completions)
                    local shells=(bash zsh fish powershell)
                    _describe 'shell' shells
                    ;;
            esac
            ;;
    esac
}
compdef _bro bro
"#.to_string()
}

fn fish_script() -> String {
    r#"# bro fish completion — source this or place in ~/.config/fish/completions/bro.fish:
# bro completions fish | source

function __bro_no_subcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __bro_seen_subcommand_from
    set -l cmd (commandline -opc)
    for sub in $argv
        if contains -- $sub $cmd
            return 0
        end
    end
    return 1
end

function __bro_aliases
    bro --complete (commandline -ct) 2>/dev/null
end

complete -c bro -f
complete -c bro -n __bro_no_subcommand -a "add"         -d "Add an alias"
complete -c bro -n __bro_no_subcommand -a "update set"  -d "Update an alias"
complete -c bro -n __bro_no_subcommand -a "remove rm"   -d "Remove an alias"
complete -c bro -n __bro_no_subcommand -a "list ls"     -d "List all aliases"
complete -c bro -n __bro_no_subcommand -a "info"        -d "Show alias details"
complete -c bro -n __bro_no_subcommand -a "search"      -d "Search aliases"
complete -c bro -n __bro_no_subcommand -a "edit"        -d "Open store in \$EDITOR"
complete -c bro -n __bro_no_subcommand -a "init"        -d "Emit shell wrapper"
complete -c bro -n __bro_no_subcommand -a "paths"       -d "Show store locations"
complete -c bro -n __bro_no_subcommand -a "run"         -d "Run alias explicitly"
complete -c bro -n __bro_no_subcommand -a "completions" -d "Emit completion script"
complete -c bro -n __bro_no_subcommand -a "(__bro_aliases)"
complete -c bro -n "__bro_seen_subcommand_from run info update set remove rm edit" -a "(__bro_aliases)"
complete -c bro -n "__bro_seen_subcommand_from init completions" -a "bash zsh fish powershell"
"#.to_string()
}

fn powershell_script() -> String {
    r#"# bro PowerShell completion — add to $PROFILE:
# Invoke-Expression (& bro completions powershell | Out-String)
Register-ArgumentCompleter -Native -CommandName bro -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $tokens = $commandAst.CommandElements
    $subcmds = @('add','update','set','remove','rm','list','ls','info','search','edit','init','paths','run','completions')

    if ($tokens.Count -le 2) {
        $aliases = & bro --complete $wordToComplete 2>$null
        ($subcmds + $aliases) | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    } elseif ($tokens.Count -eq 3 -and $tokens[1].Value -in @('run','info','update','set','remove','rm','edit')) {
        $aliases = & bro --complete $wordToComplete 2>$null
        $aliases | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    } elseif ($tokens.Count -eq 3 -and $tokens[1].Value -in @('init','completions')) {
        @('bash','zsh','fish','powershell') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
}
"#.to_string()
}
