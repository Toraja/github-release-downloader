## Context

`github-latest-release-downloader` is a single-file Rust CLI tool that fetches the latest release from a GitHub repo and downloads a matched asset to disk. Currently there is no post-download processing — the asset is always saved as-is.

Many GitHub release assets are `.tar.gz` archives containing a single binary. Users routinely follow a download with a manual `tar xzf` invocation to get the binary. Adding an `--extract` flag closes that gap and makes the tool self-contained for the common install-a-binary workflow.

## Goals / Non-Goals

**Goals:**
- Add `--extract` flag to extract the downloaded archive in place of saving it
- Support `.tar.gz` format (gzip-compressed tar)
- Delete the archive after successful extraction
- Print the destination directory path on successful extraction
- Reject `--output` + `--extract` combinations at argument-parse time

**Non-Goals:**
- Other archive formats (`.zip`, `.tar.bz2`, `.tar.xz`, etc.)
- Selective file extraction (`--pick`, `--strip-components`)
- Keeping the archive alongside extracted contents

## Decisions

### 1. Crates: `flate2` + `tar`

Use `flate2` for gzip decoding and `tar` for archive unpacking. Both are the de facto standard crates for this in the Rust ecosystem, actively maintained, and have no unsafe surface exposed to callers.

**Alternative considered:** Shell out to the system `tar` binary. Rejected — not portable (Windows), adds process-spawn complexity, and makes error handling harder to unify with the existing style.

### 2. Stream HTTP response directly into the tar extractor

The HTTP response body implements `Read`. Feed it through a `flate2::read::GzDecoder` into a `tar::Archive` and call `unpack(dest_dir)`. No temp file is needed.

**Alternative considered:** Download to a temp file first, then extract. Adds unnecessary disk I/O and complexity; direct streaming fits naturally into the tool's linear pipeline.

### 3. `--output` conflicts with `--extract` via clap `conflicts_with`

Enforced at argument-parse time, before any HTTP request. `--output` names a single file path; extraction produces a directory of files — semantically incompatible.

**Alternative considered:** Treat `--output` as the extraction directory when `--extract` is set. Rejected — conflates two distinct semantics on the same flag and is surprising to users.

### 4. Format detection by file extension

Detect `.tar.gz` (and equivalently `.tgz`) from the asset filename. If `--extract` is specified and the asset does not have a recognised extension, exit with an error before downloading.

**Alternative considered:** Detect by magic bytes after downloading. More robust but requires downloading before failing; extension-based check fails fast with a clear error message.

### 5. Success message prints destination directory

When `--extract` is used, print `Extracted to: <dir>` regardless of how many files were extracted. This is always knowable before extraction and is consistent across single-file and multi-file archives.

## Risks / Trade-offs

- **Partial extraction on failure** → If extraction fails mid-stream, already-written files are not cleaned up. Mitigation: acceptable for initial implementation; cleanup can be added later if it becomes a pain point.
- **Archive with path traversal entries** → The `tar` crate skips absolute paths and `..` components by default when using `unpack()`, so this is mitigated by the library.
- **`.tgz` alias** → Treat `.tgz` identically to `.tar.gz`. Low risk, small scope.
