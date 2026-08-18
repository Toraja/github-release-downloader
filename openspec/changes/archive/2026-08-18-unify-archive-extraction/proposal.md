## Why

Extraction currently has two independent code paths (whole archive vs. single entry) that `run()` must branch between, pushing archive-specific dispatch into the main module. The single-entry flag (`--extract-entry`) is also modelled as an *alternative* to `--extract` rather than a *qualifier* of it, which is conceptually backwards: selecting an entry is a narrowing of "extract this archive", not a separate action. Unifying both paths behind one archive entry point and reshaping the flags to match makes the main module thinner and the CLI model coherent.

## What Changes

- Rename `--extract-entry` (`-X`) to `--archive-entry`. **BREAKING** (flag name change).
- Make `--archive-entry` a qualifier of `--extract` instead of an alternative:
  - Remove the mutual exclusion between the entry flag and `--extract`.
  - `--archive-entry` now **requires** `--extract` (declarative clap `requires`).
  - `--extract` alone = whole archive; `--extract --archive-entry PATH` = single entry.
- Relax the `--output` / `--extract` conflict from unconditional to conditional:
  - `--output` remains rejected for **whole-archive** extraction (`--extract` without `--archive-entry`).
  - `--output` is **allowed** with `--extract` when `--archive-entry` is present (renaming a single extracted entry).
  - This conditional rule cannot be expressed by clap declaratively, so it is enforced by a small post-parse validation that emits a clap-style `ArgumentConflict` error with a targeted message.
- Unify the archive module behind a single entry point that takes an optional entry argument (`None` = whole archive, `Some(path)` = single entry). This is an internal refactor; existing per-mode behaviour (file rename via `--output`, directory rename, `--dir` fallback, symlink handling, whole-archive unpack) is preserved unchanged.

## Capabilities

### New Capabilities

*(none)*

### Modified Capabilities

- `extract-entry`: The flag is renamed from `--extract-entry` to `--archive-entry`; it is no longer mutually exclusive with `--extract` but instead requires it. All file/directory extraction, renaming, symlink, and not-found behaviours are otherwise preserved under the new flag name.
- `asset-extraction`: `--extract` may now be combined with `--archive-entry` to narrow extraction to a single entry; `--extract` alone continues to unpack the whole archive.
- `download-location`: The `--output` / `--extract` mutual exclusion becomes conditional — `--output` is rejected only for whole-archive extraction and is permitted when `--archive-entry` is also present.

## Impact

- `src/main.rs`: rename the flag on `Args`; replace `--extract-entry` `conflicts_with = "extract"` with `--archive-entry` `requires = "extract"`; remove `--extract` `conflicts_with = "output"`; add post-parse validation rejecting `--extract` + `--output` without `--archive-entry`; collapse the two extraction branches in `run()` into one call.
- `src/archive.rs`: unify `extract_archive` and `extract_archive_entry` behind one entry point taking `Option<&str>` for the entry and a destination type distinguishing `--dir` (directory) from `--output` (verbatim path).
- Tests: update flag-conflict tests for the new name and conditional rule; add a case for whole-archive + `--output` rejection and `--extract` + `--archive-entry` + `--output` acceptance.
- Docs: `README.md` and flag help text reflect the renamed flag and new combination rules.
