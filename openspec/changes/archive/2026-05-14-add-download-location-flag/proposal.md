## Why

Currently the tool always saves downloaded assets to the current working directory, giving users no control over where the file lands. Users who run the tool from a different directory or want to save to a specific path must manually move the file afterward.

## What Changes

- Add an optional `--dir` / `-D` flag to the CLI accepting a destination directory path
- Add an optional `--output` / `-O` flag to the CLI accepting an exact file path (enabling rename)
- `--dir` and `--output` are mutually exclusive
- When neither flag is given, the asset is saved to the current working directory with its original filename (default behaviour preserved)
- Parent directories are created automatically when either flag is used

## Capabilities

### New Capabilities

- `download-location`: Accept `--dir` and `--output` flags (mutually exclusive) to control where the downloaded asset is saved

### Modified Capabilities

- `asset-download`: The download step gains an output path parameter that determines the save location

## Impact

- `src/main.rs` (or wherever `Cli` and download logic live): add `--dir` and `--output` flags to `Cli` struct, mark them mutually exclusive, and thread the resolved path through to the file-write step
- No breaking changes — the flag is optional with existing default behaviour
