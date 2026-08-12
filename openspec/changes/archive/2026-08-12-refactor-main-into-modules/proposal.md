## Why

All application logic lives in a single `src/main.rs` file. As functionality grows, this makes navigation, testing, and reasoning about responsibilities harder. The file also has repeated HTTP setup code and inconsistent error representations.

## What Changes

- Split `src/main.rs` into focused modules: `github`, `archive`, `output`
- Introduce `thiserror`-based `AppError` enum to replace ad-hoc `String` errors
- Extract a shared `fetch_asset()` HTTP primitive so `download`, `extract_asset`, and `extract_entry` no longer duplicate HTTP setup
- Rename the file-writing function to `save_to_file` to accurately reflect its responsibility (it writes bytes from a reader, not necessarily the network)
- Extract `run() -> Result<(), AppError>` from `main()` to eliminate repeated `eprintln!` / `exit(1)` boilerplate

## Capabilities

### New Capabilities

- `module-layout`: Code is split across `src/github.rs`, `src/archive.rs`, `src/output.rs`, and a thin `src/main.rs`
- `typed-errors`: All fallible functions return `Result<_, AppError>` using `thiserror`
- `dry-asset-fetch`: A single `fetch_asset()` function handles HTTP download setup; action functions receive `impl Read`

### Modified Capabilities

<!-- No existing spec-level behavior changes — this is a pure internal refactor. -->

## Impact

- `src/main.rs`: reduced to CLI struct, `main()`, and `run()`
- New files: `src/github.rs`, `src/archive.rs`, `src/output.rs`
- `Cargo.toml`: add `thiserror` dependency
- Public behavior and CLI interface are unchanged
- All existing tests continue to pass without modification
