# bro-cli (Rust) — Extensions

Deferred features. None of these require reworking the core *if* the four seams from
`EXECUTION_PLAN.md` are in place:

1. **serde forward-compat** — no `deny_unknown_fields`; new `Alias` fields are
   `#[serde(default, skip_serializing_if = ...)]`.
2. **`exec::substitute_args`** — the single function arg-handling flows through.
3. **`config::state_path()`** — reserved file for mutable state, separate from the alias store.
4. **`Shell` trait + `InjectionMode`** — new shells/backends are additive.

Each section below says exactly where it hooks in and how big the change is.

---

## 1. Argument placeholders  ·  *priority: high*

Today extra args are  *appended* . Add named/positional substitution:

```toml
[aliases]
deploy = "kubectl apply -f {} -n {ns}"
```

```
bro deploy manifest.yaml --ns vphs-grants
```

* `{}` / `{1}`, `{2}` → positional; `{name}` → from `--name value`.
* If a template contains no placeholders, fall back to current append behavior.
* Always `shell.quote()` substituted values.

**Hook:** `exec::substitute_args` only. Model unchanged. This is the biggest real
ergonomics win and the reason that function was isolated in Phase 8.

---

## 2. `bro edit [alias]`  ·  *priority: high*

Open the store (or jump to one alias) in `$EDITOR` / `$VISUAL`. Trivial now that the store is
a hand-editable text file — this is most of the payoff of dropping SQLite.

* `bro edit` → open global store; `bro edit --local` → project `.bro`.
* `bro edit <alias>` → open store, optionally seek to the line.
* Re-validate (TOML parse) on save; reject + reopen on parse error.

**Hook:** new `commands/edit.rs`, uses existing `config` paths. Additive.

---

## 3. `--dry-run` / `bro -n`  ·  *priority: high (nearly free)*

Print the fully resolved, arg-substituted command **without** running or emitting it. Critical
for debugging the eval/quoting path.

**Hook:** one branch in `exec::run` — build `cmd_str`, print to stderr, skip emit/exec.

---

## 4. Tab completion  ·  *priority: high*

Single biggest daily-driver multiplier.

* Static scaffolding via `clap_complete` (`bro completions <shell>`).
* Dynamic alias-name completion via a hidden `bro --complete <prefix>` the completion script
  calls, returning matching names from the merged store.

**Hook:** additive command + hidden flag. No core change.

---

## 5. Tags / groups  ·  *priority: medium*

```toml
foo = { cmd = "...", tags = ["vphs", "k8s"] }
```

```
bro ls --tag vphs
```

**Hook:** add `tags: Vec<String>` to `Alias` with `#[serde(default)]`; filter in `list`.
One field + one filter. (This is exactly the forward-compat rule paying off.)

---

## 6. Usage stats / `ls --by-usage`  ·  *priority: medium*

Track run counts and last-used. **Keep it out of the alias store** — write to
`config::state_path()` (reserved in Phase 1) so the alias TOML stays clean, diffable, and
hand-editable. Do not reintroduce a database for this.

* Increment on each `run` (best-effort, ignore write failures).
* `bro ls --by-usage` joins counts at display time.

**Hook:** `exec::run` bump + separate state file. Store schema untouched.

---

## 7. Interactive picker  ·  *priority: medium*

`bro` with no args (or `bro -f`) → fuzzy-pick an alias, then run it.

* Shell out to `fzf` if present; else fall back to a Rust fuzzy matcher (`nucleo`).
* Selected alias goes through the normal `run` path (so cd/activate still work via the
  wrapper).

**Hook:** new dispatch for the no-arg case; reuses `run`. Additive.

---

## 8. confirm-before-run  ·  *priority: medium*

Guard destructive aliases:

```toml
nuke = { cmd = "kubectl delete ns {ns}", confirm = true }
```

* On run, prompt on **stderr/tty** before emitting. In the wrapped path, the prompt must
  happen in the binary (stderr/tty) *before* printing the eval code.

**Hook:** `confirm: bool` field (serde default) + a guard at the top of `exec::run`.

---

## 9. cmd.exe support  ·  *priority: medium (already wired)*

The core already routes through `Shell` + `InjectionMode::TempFileCall`. Implementing cmd =
filling in `shell/cmd.rs`:

* `init_script` emits a `bro.bat` wrapper that calls
  `bro.exe --emit --shell cmd --exec-file "%TEMP%\bro_%RANDOM%.bat" run %*`, then
  `call`s the temp file and deletes it (this is the proven Python-era mechanism, ported).
* `quote` = cmd-style quoting; `sequence` joins with `&` / newlines into the one temp `.bat`
  (whole chain to one file preserves order — same reasoning as Phase 10).
* `injection_mode` = `TempFileCall` (already returned by the stub).

**Hook:** one file. `exec::run` already branches on `InjectionMode`, so no core change.
This is the payoff of treating "where emitted code goes" as the only per-shell variable.

---

## 10. Cross-shell alias translation  ·  *priority: low / maybe never*

Aliases are stored shell-native (`cd x && ...`). Running a bash-authored alias under
PowerShell would need translating `&&`→`;`, `cd`→`Set-Location`, etc. — a large surface for
marginal benefit, since most users live in one shell family per machine.

**Recommendation:** treat as out of scope unless a concrete need appears. If it ever ships, it
slots behind the existing `Shell` trait (add a `translate` method); still additive, but the
parsing/normalizing cost is real. Documented here so it's a conscious non-goal, not an
oversight.

---

## Suggested build order after the core ships

1. Placeholders + `--dry-run` + `edit` (small, high daily value, all isolated hooks).
2. Tab completion (multiplier).
3. cmd.exe (already wired; unblocks the third shell).
4. Tags + usage stats (organization, once the alias count grows).
5. Picker + confirm (polish).
6. Cross-shell translation only if genuinely needed.
