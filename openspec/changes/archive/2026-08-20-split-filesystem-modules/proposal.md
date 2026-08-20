## Why

`src/archive.rs` has accreted responsibilities beyond archive extraction: it also owns `Destination` (destination-path resolution) and `save_to_file` (writing a plain byte stream to disk), neither of which is archive-specific. This blurs the module's purpose and gives filesystem side-effects no clear home — a problem made concrete by the upcoming `--executable` feature, whose `set_executable` helper has nowhere natural to live. Splitting these concerns first keeps that feature change focused.

## What Changes

- Introduce `src/destination.rs` holding the `Destination` enum and its `resolve` function (neutral destination-path vocabulary shared by both the download and extract paths).
- Introduce `src/fs.rs` holding `save_to_file` (filesystem side-effect: write a stream to a resolved path).
- Reduce `src/archive.rs` to tar.gz extraction only (`is_extractable`, `extract_archive`, `unpack_tar_gz`, `extract_archive_entry`, `normalize_entry_path`), importing `Destination` from `destination`.
- Update `src/main.rs` imports accordingly.
- Move the associated unit tests to the modules that now own the code.
- No behavior change: this is a pure module reorganization. Public CLI behavior, flags, output, and error handling are unchanged.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
<!-- None. Pure internal refactor with no spec-level behavior change; skip_specs is set in .openspec.yaml. -->

## Impact

- `src/archive.rs`: `Destination`/`resolve` and `save_to_file` (and their tests) moved out; retains extraction logic; adds `use crate::destination::Destination`.
- `src/destination.rs` (new): `Destination` enum + `resolve` + their tests.
- `src/fs.rs` (new): `save_to_file` + its tests.
- `src/main.rs`: `mod destination;`, `mod fs;`; update `use` statements (`Destination` from `destination`, `save_to_file` from `fs`, extraction items from `archive`).
- No changes to `src/error.rs`, `src/github.rs`, dependencies, or the CLI surface.
- Establishes the module boundaries the `add-executable-flag` change depends on (its `set_executable` helper will live in `fs.rs`).
