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
  store/
    mod.rs         Store::load / Store::save (atomic rename)
    model.rs       Alias struct + AliasEntry untagged enum
    toml_store.rs  RawStore serde target
  shell/
    mod.rs         ShellKind, InjectionMode, Shell trait, registry()
    posix.rs       bash / zsh / fish impl
    powershell.rs  PowerShell impl
    cmd.rs         cmd.exe stub (TempFileCall wired, impl pending)
  exec/
    mod.rs         emit_or_exec core, substitute_args, run_one, run_chain
  commands/
    add.rs  update.rs  remove.rs  list.rs  info.rs  search.rs  init.rs
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
| cmd.exe *(pending)* | `TempFileCall` | binary writes to temp `.bat`; wrapper `call`s it |

### Alias store

Plain TOML — no SQLite, hand-editable, diff-friendly. Atomic saves (write tempfile → rename). Forward-compatible: no `deny_unknown_fields`, new fields use `#[serde(default)]`.

```toml
[aliases]
gs   = "git status"                              # plain string shorthand
proj = { cmd = "cd ~/myapp", shell = true }      # full form
```

### Resolution order

1. Project `.bro` (nearest ancestor directory) — shadows global
2. Global `~/.config/bro/aliases.toml`

`bro info <alias>` warns when a global alias is being shadowed.

### Classifier (`classify.rs`)

`is_stateful(cmd, shell)` splits on `&&`, `||`, `;`, `|`, `\n` and checks each segment's first token against per-shell stateful sets (POSIX: `cd export source .` etc.; PowerShell: `Set-Location $env:` assignments etc.). Multi-word specials: `conda activate`, `nvm use`, `pyenv shell`. Explicit `shell: true/false` on an alias overrides detection entirely.

---

## Deprecated

`deprecated/` holds the original Python implementation (`bro.py`, `db.py`, `config.py`, etc.). Kept for reference only.
