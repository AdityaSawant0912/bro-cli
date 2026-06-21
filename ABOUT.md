# Bro CLI — Architecture

Rust rewrite of the original Python alias manager. See [`EXECUTION PLAN.md`](EXECUTION%20PLAN.md) for the build phases and [`EXTENSIONS.md`](EXTENSIONS.md) for planned features.

---

## Module map

```
src/
  main.rs          parse args → dispatch
  cli.rs           clap definitions (Cli, Cmd, *Args structs)
  config.rs        path resolution ($BRO_CONFIG, project .bro discovery)
  resolve.rs       project → global alias shadowing
  classify.rs      is_stateful() — detects cd/export/source/etc per shell
  stats.rs         UsageState — run counts + last-used in state.toml
  store/
    mod.rs         Store::load / Store::save (atomic rename)
    model.rs       Alias struct + AliasEntry untagged enum
    toml_store.rs  RawStore serde target
  shell/
    mod.rs         ShellKind, InjectionMode, Shell trait, registry()
    posix.rs       bash / zsh / fish impl
    powershell.rs  PowerShell impl
    cmd.rs         cmd.exe impl (TempFileCall — bro.bat wrapper)
  exec/
    mod.rs         emit_or_exec core, substitute_args (placeholders), run_one, run_chain
  commands/
    add.rs  update.rs  remove.rs  list.rs  info.rs  search.rs
    init.rs  paths.rs  edit.rs  completions.rs  pick.rs
```

---

## Key design decisions

### Why a shell wrapper?

A child process cannot mutate its parent shell's state (`cd`, `export`, `source`, venv activate). `bro` solves this by emitting shell code on stdout that the wrapper function `eval`s in the live shell:

```
bro <alias>  →  resolve  →  emit shell code on stdout  →  wrapper evals it
```

Invariants:
- **stdout = shell code only** in `--emit` mode. Nothing else.
- **stderr = all human messages** (errors, warnings, hints).
- **Nonzero exit → no stdout** — the wrapper checks exit code before eval.

### InjectionMode

| Shell | Mode | Mechanism |
|-------|------|-----------|
| bash / zsh / fish / PowerShell | `EvalStdout` | wrapper captures stdout, `eval`s it |
| cmd.exe | `TempFileCall` | binary writes to temp `.bat`; wrapper `call`s it |

### Alias store

Plain TOML — no SQLite, hand-editable, diff-friendly. Atomic saves (write tempfile → rename). Forward-compatible: no `deny_unknown_fields`, new fields use `#[serde(default)]`.

```toml
[aliases]
gs     = "git status"
proj   = { cmd = "cd ~/myapp", shell = true }
deploy = { cmd = "kubectl apply -f {} -n {ns}", tags = ["k8s"], confirm = true }
```

The `Alias` struct:

```rust
pub struct Alias {
    pub cmd:     String,
    pub shell:   Option<bool>,      // None = auto-detect
    pub desc:    Option<String>,
    pub tags:    Vec<String>,       // freeform categories
    pub confirm: Option<bool>,      // y/N prompt before run
}
```

Plain-string shorthand (`gs = "git status"`) is used whenever `shell`, `desc`, `tags`, and `confirm` are all at their defaults.

### Resolution order

1. Project `.bro` (nearest ancestor directory) — shadows global
2. Global `~/.config/bro/aliases.toml`

`bro info <alias>` warns when a global alias is being shadowed.

### Arg substitution (`exec::substitute_args`)

If the command template contains `{}`, `{N}`, or `{name}` placeholders, they are substituted from extra args:

- `{}` — next positional arg (auto-numbered)
- `{1}`, `{2}` — explicit 1-indexed positional
- `{name}` — from `--name value` in extra args

If no placeholders are present, extra args are appended (original behavior). Substituted values are always `shell.quote()`d.

### Interactive picker (`commands/pick.rs`)

`bro` with no args (or `bro -f`) → interactive fuzzy picker. `pick` is a hidden internal subcommand — not user-facing, called by the wrappers.

**Priority:** fzf (subprocess) → `dialoguer::FuzzySelect` (built-in, no extra install).

The wrappers route no-arg/`-f` to `bro --emit pick` so the selected alias's shell code is eval'd in the live shell — stateful aliases (cd, venv activate) persist. fzf opens `/dev/tty` for its UI so binary's stdout stays clean for the wrapper to capture. `dialoguer` handles the built-in case with a scrollable fuzzy-filtered list in the terminal.

### Usage tracking (`stats.rs`)

Run counts and last-used timestamps are written to `config::state_path()` (`state.toml`, sibling of `aliases.toml`) on every successful exec — best-effort, write failures are silently ignored. The alias store is never touched, keeping it clean and diff-friendly.

### Classifier (`classify.rs`)

`is_stateful(cmd, shell)` splits on `&&`, `||`, `;`, `|`, `\n` and checks each segment's first token against per-shell stateful sets (POSIX: `cd export source .` etc.; PowerShell: `Set-Location $env:` assignments etc.). Multi-word specials: `conda activate`, `nvm use`, `pyenv shell`. Explicit `shell: true/false` on an alias overrides detection entirely.

---

## Deprecated

`deprecated/` holds the original Python implementation (`bro.py`, `db.py`, `config.py`, etc.). Kept for reference only.
