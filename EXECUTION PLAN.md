# bro-cli (Rust) — Execution Plan

Build plan for the core. Goal: ship a working `bro` that matches the Python version
(add/update/remove/list/info/search/chain, global + project aliases) **plus** correct
`cd`/venv-activate behavior, on **UNIX shells and PowerShell**, with **cmd.exe a drop-in
addition later**.

Extensions live in `EXTENSIONS.md`. Each phase below calls out the seam an extension
hooks into so the core never has to be reworked.

---

## The one design decision everything hangs on

A child process cannot mutate its parent shell. So **the binary does not run
shell-stateful commands itself.** Instead:

```
bro <alias>  →  resolve alias  →  produce shell code on STDOUT  →  wrapper eval's it
```

The user installs a tiny shell function (also named `bro`) once. That function captures
the binary's stdout and `eval`s it in the *live* shell — so `cd`, `source venv/bin/activate`,
`$env:X=...` all persist.

Invariants that make this safe and shell-agnostic:

- **stdout = shell code only.** Nothing else is ever written to stdout in `--emit` mode.
- **stderr = human messages.** Errors ("alias not found"), warnings, info — all stderr.
- **Nonzero exit = print nothing to stdout.** The wrapper checks status and returns
  *before* eval, so a failed resolve never eval's garbage.
- **Pure vs stateful is irrelevant to the protocol.** *Everything* a wrapped call resolves
  to is emitted and eval'd in the parent shell — pure commands stream fine, stateful ones
  persist. The classifier (Phase 6) only matters for the *unwrapped fallback* and for
  warnings.

**Unwrapped fallback:** if `--emit` is absent (user called the raw binary, no wrapper
installed), the binary executes pure commands directly (`std::process::Command`, inherited
stdio) and prints a one-line stderr warning for stateful ones telling them to run
`bro init`. Graceful degradation, no panic.

**Why cmd.exe is then just an addition:** cmd can't `eval` a captured variable cleanly, but
it can `call` a `.bat`. So the *only* per-shell difference is **where emitted code goes** —
stdout (eval) vs a temp file (call). That's one enum (`InjectionMode`) and one extra `Shell`
impl. No core change.

---

## Phase 0 — Project skeleton

1. `cargo new bro --bin`. Binary name stays `bro`; the wrapper *function* is also `bro` and
   calls the binary by **absolute path** (baked in at `init` time via `std::env::current_exe`),
   so there is no PATH collision and no renaming.
2. `Cargo.toml` dependencies (no versions pinned here — take current stable):
   - `clap` (derive feature) — CLI
   - `serde` (derive) + `toml` — store
   - `shell-words` — quote-aware tokenizing (classifier)
   - `shlex` — POSIX-safe quoting when emitting (PowerShell/cmd quoting handled per-shell)
   - `directories` — default config path (XDG / `%APPDATA%`)
   - `tempfile` — atomic store writes (and cmd's temp `.bat` later)
   - `anyhow` — error plumbing
3. Module skeleton (create empty files so phases drop in cleanly):

```
src/
  main.rs            # parse args → dispatch
  cli.rs             # clap definitions
  config.rs          # path resolution ($BRO_CONFIG, project .bro discovery)
  store/
    mod.rs           # load / save (atomic)
    model.rs         # Alias + metadata (forward-compatible)
    toml_store.rs    # (de)serialization
  resolve.rs         # project → global shadowing
  classify.rs        # is_stateful()
  shell/
    mod.rs           # Shell trait + registry + InjectionMode
    posix.rs         # bash / zsh / fish
    powershell.rs
    cmd.rs           # stub for now (returns "not yet supported")
  exec/
    mod.rs           # emit-and-eval core + unwrapped fallback
  commands/
    mod.rs
    add.rs  remove.rs  list.rs  info.rs  search.rs  init.rs
```

**Done when:** `cargo build` compiles the empty skeleton.

---

## Phase 1 — Config & paths (`config.rs`)

Replaces `BRO_CLI_PATH` with **`$BRO_CONFIG`**.

- Global store path resolution, in order:
  1. `$BRO_CONFIG` if set. If it points to a file, use it. If it points to a directory,
     use `<dir>/aliases.toml`.
  2. Else `directories::ProjectDirs` → `~/.config/bro/aliases.toml` (Linux),
     `~/Library/Application Support/bro/aliases.toml` (mac),
     `%APPDATA%\bro\aliases.toml` (Windows).
- Create the file (and parent dirs) on first write, not on read.
- Project store discovery: walk up from CWD to filesystem root, return the **nearest**
  `.bro`. Cache the result for the process.
- Expose: `global_store_path() -> PathBuf`, `project_store_path() -> Option<PathBuf>`.

**Extension seam:** add a sibling `state_path()` (e.g. `<config dir>/state.toml`) now, return
the path, leave it unused. Usage-stats (EXTENSIONS) write there so the alias store stays
hand-editable and never churns.

**Done when:** a unit test sets `$BRO_CONFIG` to a tempdir and gets the expected paths;
unset falls back to `directories`.

---

## Phase 2 — Store model & I/O (`store/`)

`model.rs` — the unified alias. The Python `cmd`/`python`/`javascript`/`powershell` tables
collapse into one: an alias is **a command template plus metadata**. `--py`/`--js` are
*add-time sugar* that expand to `python <path>` / `node <path>` (Phase 5).

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alias {
    pub cmd: String,                 // the command template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<bool>,         // None = auto-detect; Some(true/false) = override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    // EXTENSION FIELDS go here later (tags, confirm, ...). See below.
}
```

**Forward-compatibility rule (critical for "minimal change"):** do **not** use
`#[serde(deny_unknown_fields)]`. serde ignores unknown keys by default, so an older binary
reading a newer file (with `tags`, `confirm`, etc.) won't break. New fields are added with
`#[serde(default, skip_serializing_if = ...)]` so the TOML stays clean for aliases that don't
use them.

On-disk shape (simple aliases stay one-liners via inline-table coercion):

```toml
[aliases]
gs   = "git status"
proj = { cmd = "cd ~/UB && code .", shell = true }
cphw = { cmd = "python ~/bro/py_scripts/cp_hw.py", desc = "copy HW template" }
```

`mod.rs` — load/save:
- `load(path) -> Store` : parse TOML; tolerate a bare string value (`gs = "..."`) by
  deserializing into `cmd` with defaults (custom `Deserialize` or a `#[serde(untagged)]`
  enum `Plain(String) | Full(Alias)` normalized on load).
- `save(path, store)` : **atomic** — write to a `NamedTempFile` in the same directory, flush,
  `persist`/rename over the target. Never truncate-in-place.
- Same `Store` type serves global and project; project just omits global-only concerns.

**Done when:** round-trip test (load → mutate → save → reload) preserves data, and a
hand-written file with mixed plain/inline-table aliases parses.

---

## Phase 3 — Resolution (`resolve.rs`)

- `resolve(name) -> Option<Resolved>` where `Resolved { alias: Alias, source: Source }` and
  `Source { Project(PathBuf), Global }`.
- Order: project `.bro` first; if found, use it and skip global (shadowing). Else global.
- `resolve_with_shadow_info(name)` returns whether a global alias is being shadowed, so
  `info` (Phase 5) can warn.

**Done when:** with a project alias and a same-named global alias, resolve returns the
project one and reports the shadow.

---

## Phase 4 — CLI skeleton (`cli.rs`, `main.rs`)

Real subcommands **plus** `bro <alias> [args...]` as the default. clap's external-subcommand
catch-all gives both:

```rust
#[derive(Parser)]
#[command(name = "bro")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Add(AddArgs),
    #[command(visible_alias = "rm")] Remove(RemoveArgs),
    #[command(visible_alias = "ls")] List(ListArgs),
    Info(InfoArgs),
    Search(SearchArgs),
    Init(InitArgs),                       // emit shell wrapper
    Run(RunArgs),                         // explicit run / chaining
    #[command(external_subcommand)]
    External(Vec<String>),                // `bro <alias> args...` lands here
}
```

- `External(v)` → treat `v[0]` as alias name, `v[1..]` as extra args → run path (Phase 8).
- A global `--emit` flag and `--shell <name>` flag live on `Cli` (set by the wrapper).
  Also a global `--exec-file <path>` (used only by cmd's `InjectionMode::TempFileCall`; inert
  otherwise).
- **Collision note:** an alias literally named `list`/`add`/`init` is shadowed by the
  subcommand. Document `bro run <alias>` as the escape hatch and ship it from day one.

**Done when:** `bro foo a b` dispatches to External with `["foo","a","b"]`; `bro list`
dispatches to List.

---

## Phase 5 — Manage commands (`commands/`)

- `add`: flags `--py`, `--js`, `--shell`/`--no-shell`, `--desc`, `--local`.
  - `--py PATH` expands to `cmd = "python <PATH>"`; `--js PATH` → `node <PATH>`; plain
    positional → stored as-is. (Expansion table lives in one place so adding `--deno` later
    is a one-line addition — an EXTENSION seam.)
  - `--shell` sets `shell = Some(true)`, `--no-shell` → `Some(false)`, neither → `None`
    (auto-detect).
  - `--local` writes to the project `.bro` (create via `init`-style template if missing),
    else global.
- `update`/`set`, `remove`/`rm`: same `--local` routing.
- `list`/`ls`: merge project + global, mark source and shadowing; columns: name, source,
  shell-flag, cmd (and `desc` if present).
- `info`: type/source/value + shadow warning from Phase 3.
- `search`: substring over names and `cmd` values.

**Done when:** full CRUD works against both global and project stores from the CLI.

---

## Phase 6 — Classifier (`classify.rs`)

`is_stateful(cmd: &str, shell: ShellKind) -> bool` — the "shell-mutating auto-detection"
feature.

- Tokenize with `shell-words` (quote-aware).
- Split into segments on `&&`, `||`, `;`, `|`, newline.
- For each segment, take the first token; return true if it's in the stateful set.
- Sets (union is fine for v1; keep per-shell so PS can differ):
  - POSIX: `cd pushd popd export set unset source . alias activate deactivate`
  - PowerShell: `cd Set-Location pushd popd Push-Location Pop-Location` + `$env:`/`$global:`
    assignment prefixes + dot-sourcing (`. `)
  - Multi-word special cases: `conda activate`, `nvm use`, `pyenv shell`, `rbenv shell`.
- Explicit `shell` field overrides detection entirely.

Note: in the wrapped/eval path this only drives *warnings and the unwrapped fallback* — eval
runs everything in-shell regardless. It becomes load-bearing for the cmd temp-file path and
for `--dry-run` labeling.

**Done when:** table-test of representative commands classifies correctly per shell.

---

## Phase 7 — Shell abstraction (`shell/`)

The extensibility spine. One trait, a registry, an injection mode.

```rust
pub enum InjectionMode { EvalStdout, TempFileCall }

pub trait Shell {
    fn kind(&self) -> ShellKind;
    fn injection_mode(&self) -> InjectionMode;
    fn quote(&self, arg: &str) -> String;              // safe single-arg quoting
    fn sequence(&self, cmds: &[String]) -> String;     // join preserving order (&& / ; / newline)
    fn init_script(&self, bin: &Path) -> String;       // wrapper emitted by `bro init`
}
```

- `posix.rs`: `quote` via `shlex`; `sequence` joins with `\n` (or `&&` for chains needing
  short-circuit); `injection_mode` = `EvalStdout`. fish is a `quote`/`init_script` variant of
  the same impl.
- `powershell.rs`: `quote` = single-quote with `''` escaping; `sequence` joins with `;` /
  newline; `injection_mode` = `EvalStdout`; `init_script` emits the PS function.
- `cmd.rs`: **stub** returning a clear "cmd.exe support not yet implemented — coming soon"
  error from `init`, and `injection_mode` = `TempFileCall`. Registered but inert. Adding real
  cmd support later = filling in this one file. **No other module changes.**
- `registry(shell_kind) -> Box<dyn Shell>` selects from `--shell`.

Wrappers emitted by `init_script` (binary path baked in as `{BIN}`):

```bash
# bash / zsh
bro() {
  local out
  out="$('{BIN}' --emit --shell bash run "$@")" || return $?
  eval "$out"
}
```

```powershell
# PowerShell
function bro {
  $code = & '{BIN}' --emit --shell powershell run @args
  if ($LASTEXITCODE -ne 0) { return }
  Invoke-Expression ($code -join "`n")
}
```

**Done when:** `bro init bash` / `bro init powershell` print correct wrappers with the real
binary path; `bro init cmd` prints the "coming soon" notice and exits nonzero.

---

## Phase 8 — Emit / run core (`exec/mod.rs`)

The heart. Used by `run` and by `External`.

```
run(name, extra_args, ctx):
  resolved = resolve(name)?                     # else: stderr error, exit 1, no stdout
  cmd_str  = substitute_args(resolved.cmd, extra_args, shell)   # append now; placeholders later
  shell    = registry(ctx.shell)

  if ctx.emit:                                  # wrapped call
      code = cmd_str                            # already a full command line
      match shell.injection_mode():
        EvalStdout    => print code to STDOUT   # wrapper eval's it
        TempFileCall  => write code to ctx.exec_file (cmd path, later)
      exit 0
  else:                                         # unwrapped fallback
      if is_stateful(cmd_str, shell):
          eprintln!("'{name}' changes shell state; run `bro init <shell>` to enable it");
          # optionally run in a subshell knowing cd won't persist
      else:
          spawn child (inherit stdio), propagate exit code
```

- `substitute_args`: v1 = append `extra_args` (each `shell.quote()`d) to the command. This is
  the **single function placeholders plug into** later (EXTENSIONS): detect `{}`/`{name}` and
  substitute instead of append. Isolating it now is what keeps that change to one function.
- All human output via `eprintln!`; stdout reserved for emitted code.

**Done when:** wrapped `bro gs` eval's `git status` in the live shell; wrapped `bro proj`
actually changes the shell's cwd; unwrapped `bro proj` warns instead of silently failing.

---

## Phase 9 — `init` command (`commands/init.rs`)

- `bro init <shell>` → `registry(shell).init_script(current_exe())` to stdout.
- Print install hint to **stderr** (so `eval "$(bro init bash)"` only consumes the script):
  bash/zsh → add `eval "$(bro init bash)"` to `~/.bashrc`/`~/.zshrc`; PowerShell →
  `Invoke-Expression (& bro init powershell | Out-String)` in `$PROFILE`.

**Done when:** sourcing the emitted script in a fresh bash and a fresh PowerShell makes
`bro <alias>` work end to end.

---

## Phase 10 — Chaining (`run -c`)

- `bro -c a,b,c` (and `bro run -c a,b,c`): resolve each, substitute args, then
  `shell.sequence([...])` into one emitted block. Because emit-and-eval already runs the whole
  block in the parent shell, **order and shared state are preserved for free** — no special
  "if any is stateful, route everything" logic needed (the old temp-bat constraint dissolves;
  it only re-appears for cmd's `TempFileCall`, where the whole block goes to one temp `.bat`).

**Done when:** `bro -c proj,build` cd's then builds in that directory, in order.

---

## Phase 11 — End-to-end verification

Manual matrix (script later):

| Case | bash/zsh | PowerShell |
|------|----------|------------|
| pure alias (`git status`) streams output, right exit code | ✓ | ✓ |
| stateful alias (`cd`) persists in shell | ✓ | ✓ |
| venv activate persists | ✓ | ✓ |
| extra args appended + quoted (spaces/special chars) | ✓ | ✓ |
| project alias shadows global | ✓ | ✓ |
| chain preserves order + state | ✓ | ✓ |
| unwrapped raw binary degrades gracefully | ✓ | ✓ |
| alias not found → stderr error, no stdout, nonzero exit | ✓ | ✓ |

---

## Phase 12 — Build & install

- `cargo build --release` → single static-ish binary.
- README quick-start: put binary on PATH, add the one `eval`/`Invoke-Expression` line to the
  rc/profile, done.
- `bro init` is the whole install story for shell integration.

---

## Module → extension map (so "soon" is cheap)

| Extension (see EXTENSIONS.md) | Hooks into | Core change |
|---|---|---|
| Arg placeholders `{}` / `{name}` | `exec::substitute_args` | one function |
| `bro edit` | new `commands/edit.rs` + `config` paths | additive |
| `--dry-run` | `exec::run` (print to stderr, skip emit/exec) | one branch |
| Tab completion | `clap_complete` + hidden `--complete` | additive |
| Tags / groups | `Alias` field (serde default) + `list` filter | one field |
| Usage stats | `config::state_path()` (already reserved) | separate file |
| fzf picker | new `commands` entry, no-arg dispatch | additive |
| confirm-before-run | `Alias` field + `exec::run` guard | one field |
| **cmd.exe** | fill in `shell/cmd.rs` + `TempFileCall` (already wired) | one file |

Build the phases in order; each ends in something runnable. The forward-compat serde rule
(Phase 2), the isolated `substitute_args` (Phase 8), the reserved `state_path` (Phase 1), and
the `Shell`/`InjectionMode` seam (Phase 7) are the four things that keep the extensions from
forcing a refactor.