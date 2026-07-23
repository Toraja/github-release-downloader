## Why

Users frequently download release assets that are compressed archives (e.g. `.tar.gz`) in order to obtain a binary inside. Without extraction support, they must run a separate `tar` command after downloading. An `--extract` flag eliminates that step and makes the tool useful end-to-end for the common "download and install a binary" workflow.

## What Changes

- Add `--extract` flag that, when specified, extracts the downloaded archive after downloading it
- Supports `.tar.gz` format only (initial implementation)
- The downloaded archive is deleted after successful extraction
- `--extract` conflicts with `--output` (extraction targets a directory, not a single file path)
  - Because a tarball (or other type of archive) can contains multiple files at the root level, the `--extract` flag implies that the download location must be a directory rather than a single file path, so it cannot be used with `--output`
- Success message changes when `--extract` is used: prints the destination directory instead of the file path

## Capabilities

### New Capabilities

- `asset-extraction`: Extracting a downloaded archive asset to a destination directory, deleting the archive afterwards, and reporting the extraction directory

### Modified Capabilities

- `asset-download`: Success output message changes when `--extract` is used (prints destination directory instead of downloaded file path)
- `download-location`: `--output` gains a new conflict: it is incompatible with `--extract`

## Impact

- `src/main.rs`: new `--extract` flag in `Cli`, new extraction logic, updated success message handling
- `Cargo.toml`: new dependency for tar/gzip extraction (e.g. `flate2`, `tar`)
