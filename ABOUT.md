# Bro CLI

Personal command alias manager for Windows. Store shell commands, Python scripts, JavaScript scripts, and PowerShell scripts under short memorable names — then run them with `bro <alias>`.

---

## What It Does

`bro` saves you from retyping long or complex commands. Instead of:

```
python R:\bro-cli\py_scripts\cp_hw.py
```

You do:

```
bro cphw
```

Aliases live in a global SQLite database (available everywhere) or in a per-project `.bro` TOML file (scoped to a directory tree). Project aliases shadow global ones of the same name.

---

## Architecture

```
bro.bat          ← Windows entry point; routes shell-state cmds through a temp .bat file
bro.py           ← Main CLI (Typer app); all command logic
db.py            ← Singleton SQLite wrapper (Database class)
config.py        ← Project-local .bro file handler (TOML via tomli/tomli_w)
constants.py     ← Table definitions, executors, validators; reads BRO_CLI_PATH env var
schema.sql       ← DB schema: cmd, python, javascript, powershell tables
main_db.db       ← SQLite database (global alias store)
py_scripts/      ← Example scripts registered as aliases
```

### Why `bro.bat` Exists

Python subprocesses can't modify the parent shell's state (`cd`, `set`, `activate`, etc.). `bro.bat` works around this by:

1. Creating a temp `.bat` file (`%TEMP%\bro_<random>.bat`)
2. Passing its path to `bro.py` via `--exec-file`
3. Shell-state commands get written to that file instead of executed
4. After Python exits, `bro.bat` `call`s the temp file, then deletes it

This lets `bro cd mydir` actually change the shell's working directory.

### Database Layer (`db.py`)

- Singleton pattern — one `Database` instance per DB key
- Lazy connection — only opens SQLite when first query runs
- Auto-initializes from `schema.sql` if DB file missing or tables incomplete
- `sqlite3.Row` factory — columns accessible by name

### Storage Tables (`schema.sql`)

| Table        | Primary Key | Value Column | Executor               |
|--------------|-------------|--------------|------------------------|
| `cmd`        | alias       | cmd          | runs as-is             |
| `python`     | alias       | path         | `python <path>`        |
| `javascript` | alias       | path         | `node <path>`          |
| `powershell` | alias       | path         | `powershell <path>`    |

### Project Config (`config.py`)

- Looks for `.bro` file by walking up from CWD to root
- Format: TOML with `[aliases]` section
- `bro --init` creates a template `.bro` in CWD
- Project aliases override global aliases with same name (shadowing)
- Only supports simple shell commands (no script type aliases)

---

## Command Reference

### Execute

```
bro <alias>                    # run alias (project first, then global)
bro <alias> extra args         # extra args appended to command
```

### Manage Global Aliases

```
bro -a <alias> <command>       # add shell command
bro -a <alias> -py ./script.py # add Python script
bro -a <alias> -js ./script.js # add JS script
bro -u <alias> <new-command>   # update
bro -d <alias>                 # delete
```

### Manage Project Aliases

```
bro --init                     # create .bro in CWD
bro --init --force             # overwrite existing .bro
bro -a <alias> <command> --local  # add to project .bro
bro -u <alias> <command> --local  # update in project .bro
bro -d <alias> --local            # delete from project .bro
```

### Discovery

```
bro -l                         # list all (project + global)
bro -i <alias>                 # show info (type, source, value)
bro -s <keyword>               # search alias names and values
```

### Chaining

```
bro -c alias1,alias2,alias3    # run aliases in sequence
```

Chaining respects shell-context routing — if any alias in the chain needs shell state, all commands are written to the temp `.bat` file to preserve order.

---

## Configuration

Requires env var `BRO_CLI_PATH` set to the `bro-cli` directory path. Used to locate `main_db.db` and `schema.sql`.

```bat
set BRO_CLI_PATH=R:\bro-cli
```

---

## Dependencies

| Package    | Purpose                          |
|------------|----------------------------------|
| typer      | CLI framework                    |
| tomli      | Read TOML `.bro` files (read)    |
| tomli_w    | Write TOML `.bro` files (write)  |
| sqlite3    | Built-in; global alias storage   |

---

## Alias Resolution Order

1. Project `.bro` file (nearest ancestor directory)
2. Global `main_db.db`

If found in project config → used, global skipped. `bro -i <alias>` warns if a global alias is being shadowed.

---

## Example Scripts in `py_scripts/`

These are sample scripts intended to be registered as aliases:

- **`cp_hw.py`** — Copies homework template files from `R:/UB/templates/HW` into CWD. Skips items that already exist.
- **`line_count.py`** — Counts lines per file in a given directory. Usage: `python line_count.py <dir>`
