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
- **Extra args** — pass arguments on the fly: `bro gs --oneline` appends to the stored command
- **Chaining** — `bro -c build,test,deploy` runs aliases in sequence, sharing shell state
- **Auto-detection** — `bro` detects stateful commands automatically; override with `--shell` / `--no-shell`
- **Human-readable store** — plain TOML, hand-editable, diff-friendly, no database
- **Fast** — single Rust binary, no runtime, no interpreter

---

## Install

### Windows (PowerShell)

**1. Build the binary**

```powershell
cd R:\bro-cli
cargo build --release
```

**2. Copy to a directory on your PATH** (or add `target\release` to PATH)

```powershell
Copy-Item target\release\bro.exe C:\Users\<you>\bin\bro.exe
```

**3. Install the PowerShell wrapper** (add to `$PROFILE`)

```powershell
# Run once — appends the wrapper to your PowerShell profile
bro init powershell | Out-String >> $PROFILE
# Or manually add:
Invoke-Expression (& bro init powershell | Out-String)
```

Restart PowerShell (or `. $PROFILE`). Done.

---

### WSL (bash / zsh)

**1. Build the Linux binary from WSL**

```bash
cd /mnt/r/bro-cli      # or wherever your repo lives
cargo build --release
```

> If Rust isn't installed in WSL: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**2. Put the binary on PATH**

```bash
mkdir -p ~/.local/bin
cp target/release/bro ~/.local/bin/bro
# make sure ~/.local/bin is in PATH (most distros do this automatically)
```

**3. Install the shell wrapper**

```bash
# bash — add to ~/.bashrc
echo 'eval "$(bro init bash)"' >> ~/.bashrc
source ~/.bashrc

# zsh — add to ~/.zshrc
echo 'eval "$(bro init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

Done. The wrapper is a one-line shell function — it captures `bro`'s stdout and `eval`s it so stateful commands persist.

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
bro add gs "git status"                    # shell command
bro add cphw --py scripts/cp_hw.py        # Python script → python <path>
bro add lint --js scripts/lint.js         # JS script → node <path>
bro add proj "cd ~/myapp && code ." --shell   # force stateful
bro add build "make -j8" --local          # project-scoped (.bro file)
```

### Run

```bash
bro gs                     # run alias
bro gs --short             # extra args appended
bro run gs                 # explicit run (escapes alias/subcommand collision)
bro run --chain gs,hi,build   # run aliases in sequence
```

### Manage

```bash
bro list                   # list all (project + global)
bro list --local           # project aliases only
bro info gs                # show source, cmd, shell flag
bro search git             # search names and commands
bro update gs "git status --short"
bro remove gs
```

### Shell wrapper

```bash
bro init bash              # print bash wrapper (eval it to install)
bro init zsh
bro init powershell
bro init fish
```

---

## How it works

A child process can't mutate its parent shell. `bro` solves this with a thin wrapper function:

```
bro <alias>
  → binary resolves alias
  → prints shell code to stdout
  → wrapper function evals it in the live shell
```

Your `PATH`, working directory, and activated environments all see the result.

The binary itself is safe to call directly without the wrapper — it falls back to executing pure commands in a subprocess and prints a warning for stateful ones.

---

## Store format

Aliases live in `~/.config/bro/aliases.toml` (or `%APPDATA%\bro\aliases.toml` on Windows). Plain text, always hand-editable:

```toml
[aliases]
gs   = "git status"
proj = { cmd = "cd ~/myapp && code .", shell = true }
cphw = { cmd = "python scripts/cp_hw.py", desc = "copy HW template" }
```

Project aliases go in a `.bro` file at the root of any directory. `bro` walks up from your CWD to find the nearest one. Project aliases shadow global ones of the same name.

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

See [`EXTENSIONS.md`](EXTENSIONS.md) for planned features — arg placeholders, `bro edit`, tab completion, fuzzy picker, usage stats, and more.
