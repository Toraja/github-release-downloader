## Why

When extracting an archive with `--extract`, the entire archive contents are unpacked. There is no way to extract only a specific file or directory from within the archive, nor to rename it at the destination. A dedicated flag addresses both needs cleanly without complicating the existing `--extract` + `--output` validation boundary.

## What Changes

- Add a new flag (e.g. `--extract-entry`) that accepts a path identifying a single file or directory within the archive to extract; only one entry is allowed per invocation.
- The new flag implies extraction — it is an alternative to `--extract`, not a companion.
- Make the new flag mutually exclusive with `--extract` (clap `conflicts_with`).
- Allow `--output` to be used alongside the new flag to rename the extracted entry at the destination.
- `--dir` continues to work with the new flag as the destination directory (when `--output` is not used).

## Capabilities

### New Capabilities

- `extract-entry`: A new CLI flag that selects a single file or directory entry from within an archive to extract, with optional renaming via `--output`.

### Modified Capabilities

*(none — no existing spec-level requirements change)*

## Impact

- `src/main.rs`: Add the new flag to the `Cli` struct; add entry-filtering logic to the extraction path; update `conflicts_with` constraints; update `unpack_tar_gz` (or add a sibling function) to unpack only the specified entry.
- Existing `--extract` behaviour is unchanged.
- The `--output` flag gains a new valid use-case (pairing with the new flag) but its definition does not change.
- Tests: add cases for entry selection, renaming, `--dir` fallback, and mutual exclusion with `--extract`.
