## Context

See `proposal.md` (Why) for motivation. Current state relevant to the design:

- `src/main.rs` `run()` branches three ways: `--extract-entry` (single entry), `--extract` (whole archive), and plain download. Each branch resolves a destination and calls a different archive function.
- `src/archive.rs` exposes two extraction functions: `extract_archive(reader, dest_dir)` (whole archive via tar's `Archive::unpack`) and `extract_archive_entry(reader, entry, dest)` (streams the tar, matches a file or directory entry, and lands it at `dest`). `extract_archive_entry` already unifies file-entry and directory-entry handling behind a single `dest: &Path` and already honours `--output` (verbatim) and `--dir` (dir + basename).
- Flag wiring today: `--extract-entry conflicts_with --extract`; `--extract conflicts_with --output`; `--dir conflicts_with --output`.

Constraint discovered during exploration: the target rule "reject `--output` only for whole-archive extraction" is a compound condition (`--extract ∧ ¬--archive-entry ∧ --output`) that clap cannot express declaratively (no conditional/`conflicts_with_unless`, and the `--output` value is a free-form path so `required_if_eq_*` cannot match it).

## Goals / Non-Goals

**Goals:**
- One public extraction entry point in `archive` taking an optional entry, so `main` no longer branches on extraction sub-mode.
- Flags that mirror the unified function: `--extract` = do extraction, `--archive-entry` = narrow to one entry.
- Preserve all current per-mode behaviour exactly (file rename, directory rename, `--dir` fallback, symlink handling, whole-archive unpack, messages).

**Non-Goals:**
- Merging the two extraction *bodies*. Only the signature/dispatch is unified; whole-archive and entry extraction keep their existing implementations internally.
- Changing `--dir` / `--output` semantics for entries (verbatim vs. dir+basename stays as-is).
- Supporting new archive formats or new short-flag assignments beyond keeping `-x` / `-X`.

## Decisions

### D1: Single entry point `extract_archive(reader, entry: Option<&str>, dest)`
Merge the two functions behind one signature. `entry = None` → whole archive; `entry = Some(path)` → single entry. Internally dispatch, not unify:

```
match entry {
    None    => unpack_whole(reader, dir)   // keeps tar's Archive::unpack
    Some(e) => extract_entry_loop(reader, e, landing)
}
```

*Why not unify the bodies:* whole-archive uses tar's `unpack` (preserves symlinks/permissions), whereas the entry loop deliberately skips symlinks. Routing whole-archive through the entry loop (empty prefix) would silently change symlink handling. Two bodies, one door keeps behaviour identical.

### D2: Destination type distinguishing `--dir` from `--output`
Introduce an enum encoding **flag provenance**, not entry kind (entry file-vs-dir is only knowable while streaming):

```
enum Destination {
    Into(PathBuf),    // from --dir (or default "."): a directory to place the natural-named result into
    Exact(PathBuf),   // from --output: use this path verbatim
}
```

Interpretation inside `archive`:

| entry     | Into(d)                    | Exact(p)                 |
|-----------|----------------------------|--------------------------|
| None      | unpack whole INTO d        | (impossible — see D4)    |
| Some(e)   | d.join(basename(e))        | p verbatim (rename)      |

*Why an enum over a plain `PathBuf`:* it moves the "dir + basename vs. verbatim" and "whole-archive dir vs. entry landing" resolution into `archive`, which is the responsibility we want out of `main`. Variant names describe semantics (`Into`/`Exact`) so they read correctly for both file and directory entries (an `--output` on a directory entry renames the directory root — still "exact").

*Alternative considered:* keep `resolve_output_path` producing a single `PathBuf` in `main`. Rejected because it leaves per-mode basename logic in `main`, which is what the change aims to remove.

### D3: `--archive-entry` requires `--extract` (declarative)
`#[arg(requires = "extract")]`. Chosen over "imply" because it is enforced by clap for free, keeps exactly one way to express each intent, and matches the qualifier framing of the rename (`archive-entry` describes *which* entry of the archive `--extract` targets). Cost: `--extract` is mandatory alongside `--archive-entry` (a few keystrokes).

### D4: Conditional `--output` conflict via post-parse validation
Remove `--extract conflicts_with --output`. Add a single manual check after parsing:

```
if extract && archive_entry.is_none() && output.is_some() {
    Args::command().error(ErrorKind::ArgumentConflict,
        "--output cannot be used when extracting a whole archive; use --dir").exit();
}
```

*Why manual:* the constraint is the compound `(--extract ∧ ¬--archive-entry ∧ --output)`, which clap cannot express — `conflicts_with` is unconditional (would also break the valid `--extract --archive-entry --output`), and `required_if_eq_all` can only match concrete values, not "`--output` is present with any path". Emitting via `Command::error(ArgumentConflict, …)` keeps native-looking UX and also yields a clearer message than the declarative route ("--output cannot be used …" vs. "--archive-entry is required"). This also makes the `Destination::Exact` + `entry = None` cell in D2 unreachable; `archive` handles it defensively (internal error / documented `unreachable!` tied to this guard).

## Risks / Trade-offs

- **Impossible enum cell (D2/D4).** `(Exact, None)` is forbidden by the CLI guard but representable in the type. → Handle defensively in `archive` with an explicit error rather than silent behaviour; comment ties it to the D4 guard.
- **Behavioural drift during the refactor.** Merging risks accidentally changing symlink or message behaviour. → D1 keeps whole-archive on tar's `unpack` and the entry loop untouched; existing tests for both paths must continue to pass unchanged.
- **Breaking flag rename.** `--extract-entry` → `--archive-entry` breaks existing scripts. → Documented as BREAKING in the proposal with migration notes in the spec deltas; single clear rename rather than an alias to avoid two supported spellings.
- **Redundant `--extract` keystroke (D3).** Users must now pass `--extract` alongside `--archive-entry`. → Accepted; the declarative `requires` and single-intent clarity outweigh the terseness cost.
