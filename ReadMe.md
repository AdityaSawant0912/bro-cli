# bro

> Your personal command alias manager — fast, shell-aware, and everywhere.

`bro` lets you save any command under a short name and run it from any shell. `cd`-ing, activating venvs, and chaining commands all work because `bro` emits shell code that your shell evaluates — no subprocess tricks.

```
$ bro add proj "cd ~/projects/myapp && code ."
$ bro proj          # actually changes your directory and opens VS Code
```

---

## Features

- **Shell-stateful commands work** — `cd`, `export`, `source venv/bin/activate`, `conda activate` all persist in your live shell session
- **Global + project aliases** — global aliases live in `~/.config/bro/aliases.toml`; drop a `.bro` file in any project root for scoped aliases that shadow global ones
- **Arg placeholders** — `bro deploy manifest.yaml --ns prod` substitutes `{}` and `{ns}` in the stored command; falls back to appending when no placeholders are present
- **Chaining** — `bro run -c build,test,deploy` runs aliases in sequence, sharing shell state
- **Dry-run** — `bro -n <alias>` prints the fully resolved command without executing it
- **Edit in place** — `bro edit [alias]` opens the store in `$EDITOR` with TOML re-validation on save
- **Tab completion** — `bro completions bash|zsh|fish|powershell` emits a completion script with dynamic alias-name completion
- **Interactive picker** — `bro` with no args (or `bro -f`) launches an interactive fuzzy picker; uses fzf if installed, else a built-in fuzzy select (`dialoguer`); selected alias runs through the normal shell-eval path so `cd`/`export` persist
- **Tags** — tag aliases for filtering: `bro list --tag k8s`
- **Usage stats** — `bro list --by-usage` sorts by run count; `bro info <alias>` shows run count and last-used time; stats live in a separate `state.toml`, never dirtying the alias store
- **Confirm-before-run** — mark destructive aliases with `confirm = true`; prompts `y/N` on stderr before executing
- **Auto-detection** — `bro` detects stateful commands automatically; override with `--shell` / `--no-shell`
- **Human-readable store** — plain TOML, hand-editable, diff-friendly, no database
- **Fast** — single Rust binary, no runtime, no interpreter
- **All shells** — bash, zsh, fish, PowerShell, cmd.exe

---

## Install

### Windows (PowerShell)

```powershell
.\install.ps1
. $PROFILE
```

Builds the release binary, copies it to `~/bin`, adds `~/bin` to your user PATH, and injects the PowerShell wrapper + tab completions into `$PROFILE`.

> Requires [Rust](https://rustup.rs) on PATH.

---

### WSL (bash / zsh / fish)

```bash
bash /mnt/r/bro-cli/install.sh
source ~/.bashrc   # or ~/.zshrc
```

Builds a Linux binary, copies it to `~/.local/bin`, detects your shell, and adds the wrapper + completions to your rc file.

> If Rust isn't installed in WSL: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Both scripts are idempotent — safe to re-run after updates.

---

### Verify

```
$ bro add hi "echo hey"
$ bro hi
hey
```

---

## Usage

### Add aliases

```bash
bro add gs "git status"                          # shell command
bro add cphw --py scripts/cp_hw.py              # Python script → python <path>
bro add lint --js scripts/lint.js               # JS script → node <path>
bro add proj "cd ~/myapp && code ." --shell      # force stateful
bro add build "make -j8" --local                 # project-scoped (.bro file)
bro add deploy "kubectl apply -f {} -n {ns}"     # arg placeholders
bro add nuke "kubectl delete ns {ns}" --confirm --tag k8s   # confirm + tag
```

### Run

```bash
bro gs                            # run alias
bro gs --oneline                  # extra args appended
bro deploy manifest.yaml --ns prod   # placeholder substitution
bro run gs                        # explicit run (escapes subcommand collision)
bro run -c build,test,deploy      # run aliases in sequence
bro -n gs                         # dry-run: print resolved command, don't exec
bro                               # interactive fuzzy picker (fzf or built-in)
bro -f                            # same, explicit shorthand
```

### Manage

```bash
bro list                          # list all (project + global)
bro list --local                  # project aliases only
bro list --tag k8s                # filter by tag
bro list --by-usage               # sort by run count
bro info gs                       # show source, cmd, shell flag, tags, confirm, usage stats
bro search git                    # search names, commands, descriptions, and tags
bro update gs "git status --short"   # update (preserves unspecified fields)
bro update deploy --tag k8s --tag infra   # replace tags
bro remove gs
bro edit                          # open global store in $EDITOR
bro edit gs                       # open the store containing 'gs', jump to line
bro edit --local                  # open project .bro
```

### Shell wrapper + completions

```bash
bro init bash              # print bash wrapper (eval it to install)
bro init zsh
bro init powershell
bro init fish
bro init cmd               # print bro.bat for cmd.exe

bro completions bash       # print bash tab-completion script
bro completions zsh
bro completions powershell
bro completions fish
```

---

## How it works

A child process can't mutate its parent shell. `bro` solves this with a thin wrapper function:

```
bro <alias>
  → binary resolves alias
  → substitutes arg placeholders
  → prints shell code to stdout
  → wrapper function evals it in the live shell
```

Your `PATH`, working directory, and activated environments all see the result.

The binary itself is safe to call directly without the wrapper — it falls back to executing pure commands in a subprocess and prints a warning for stateful ones.

For cmd.exe, stdout eval isn't available; instead the binary writes to a temp `.bat` file and the wrapper `call`s it, which preserves `cd` and `set` in the live session.

---

## Store format

Aliases live in `~/.config/bro/aliases.toml` (or `%APPDATA%\bro\config\aliases.toml` on Windows). Plain text, always hand-editable:

```toml
[aliases]
gs     = "git status"
proj   = { cmd = "cd ~/myapp && code .", shell = true }
cphw   = { cmd = "python scripts/cp_hw.py", desc = "copy HW template" }
deploy = { cmd = "kubectl apply -f {} -n {ns}", tags = ["k8s"] }
nuke   = { cmd = "kubectl delete ns {ns}", tags = ["k8s"], confirm = true }
```

Plain-string shorthand is used when `shell`, `desc`, `tags`, and `confirm` are all at defaults.

Project aliases go in a `.bro` file at the root of any directory. `bro` walks up from your CWD to find the nearest one. Project aliases shadow global ones of the same name.

Usage stats are tracked separately in `state.toml` (same directory) so the alias store stays clean and diff-friendly.

---

## Arg placeholders

| Syntax | Meaning |
|--------|---------|
| `{}` | next positional arg |
| `{1}`, `{2}` | explicit positional (1-indexed) |
| `{name}` | `--name value` from extra args |

If the command contains no placeholders, extra args are appended as before.

```bash
bro add deploy "kubectl apply -f {} -n {ns}"
bro deploy manifest.yaml --ns staging
# runs: kubectl apply -f manifest.yaml -n staging
```

---

## Configuration

| Env var | Purpose |
|---------|---------|
| `BRO_CONFIG` | Override global store location (file or directory) |

---

## Building from source

Requires Rust 1.75+.

```bash
git clone <repo>
cd bro-cli
cargo build --release
# binary at target/release/bro (or bro.exe on Windows)
```

---

## Roadmap

Remaining planned features — see [`EXTENSIONS.md`](EXTENSIONS.md) for details:

- **Cross-shell alias translation** (low priority / maybe never)
