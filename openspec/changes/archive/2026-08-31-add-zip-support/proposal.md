## Why

Many GitHub projects publish release assets as `.zip` archives (especially Windows builds), but the CLI currently rejects anything that is not `.tar.gz`/`.tgz` when `--extract` is used (issue #15). Zip support is also a prerequisite for issue #8.

## What Changes

- Support `.zip` assets for whole-archive extraction via `--extract` (streamed, archive not saved to disk, consistent with existing behaviour).
- Support `.zip` assets for single-entry extraction via `--extract --archive-entry PATH`, including file entries, directory entries, and the existing error cases (entry not found, symlink handling, parent directory creation, merging into existing destinations).
- Update the unsupported-format error message and CLI help text to list `.zip` as a supported format.
- Non-extraction download behaviour is unchanged: `.zip` assets can already be downloaded without `--extract`.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `asset-extraction`: The supported-format requirements expand from `.tar.gz`/`.tgz` only to also include `.zip`; the "Unsupported archive format" scenario changes accordingly.
- `extract-entry`: The "Archive format validation" requirement expands to accept `.zip`; entry extraction, directory extraction, symlink handling, and streaming requirements apply to `.zip` archives as well.

## Impact

- `src/archive.rs`: format detection (`is_extractable`) and extraction paths gain a zip implementation alongside the existing tar.gz one.
- `src/error.rs`: unsupported-format error message updated to mention `.zip`.
- `src/main.rs`: `--extract` and `--archive-entry` help text updated.
- `Cargo.toml`: new dependency for reading zip archives (e.g. the `zip` crate).
- Note: zip is a seek-based format (central directory at the end of the file), so true streaming is not possible the way it is for tar.gz; buffering strategy is a design decision covered in design.md.
