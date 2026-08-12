## Context

`src/main.rs` is 858 lines containing the CLI definition, GitHub API logic, archive extraction logic, output path resolution, the main entry point, and all tests. Functions share no common error type — every fallible function returns `Result<_, String>`, with format strings scattered through `map_err` calls. Three functions (`download_asset`, `extract_asset`, `extract_entry`) each independently build an identical `ureq::get` HTTP request.

## Goals / Non-Goals

**Goals:**
- Split code into focused modules with clear responsibilities
- Introduce a typed `AppError` enum via `thiserror`
- Centralise asset HTTP fetching into a single `fetch_asset()` primitive
- Keep CLI behavior and public interface identical
- Keep all existing tests passing

**Non-Goals:**
- Moving tests to separate files (deferred)
- Adding new features or changing CLI flags
- Switching HTTP client or adding async

## Decisions

### Module layout

```
src/
├── main.rs       ← Cli struct, main(), run()
├── github.rs     ← Asset, Release, to_api_url(), fetch_release(), select_asset()
├── archive.rs    ← is_extractable(), normalize_entry_path(), unpack_tar_gz(),
│                    extract_archive_entry(), save_to_file()
└── output.rs     ← resolve_output_path()
```

**Why this split:**
- `github.rs` groups everything that knows about the GitHub API shape
- `archive.rs` groups everything that processes byte streams (network-agnostic)
- `output.rs` is tiny but has clear standalone semantics; keeping it separate avoids polluting `main.rs`

**Alternative considered — two modules (`github`, `archive`):** `resolve_output_path` could live in `main.rs`, but separating it makes `main.rs` purely orchestration code.

### HTTP fetch primitive

Introduce `fetch_asset(asset: &Asset) -> Result<impl Read + '_, AppError>` in `github.rs`. This returns a streaming reader over the HTTP response body.

The three action functions become:

```
save_to_file(reader: impl Read, dest: &Path) -> Result<(), AppError>
extract_archive(reader: impl Read, dest_dir: &Path) -> Result<(), AppError>
extract_archive_entry(reader: impl Read, entry: &str, dest: &Path) -> Result<(), AppError>  // renamed from extract_entry_from_reader
```

`run()` in `main.rs` calls `fetch_asset` then passes the reader to the appropriate action function.

**Why `impl Read` not `Vec<u8>`:** Streaming avoids buffering the entire asset in memory. For large binaries this matters.

### Error type

Single `AppError` enum in `src/error.rs` (or inline in `main.rs` — see trade-off below):

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid URL: host must be github.com, got {0}")]
    InvalidHost(String),
    #[error("Invalid URL: expected owner/repo path, got {0}")]
    InvalidPath(String),
    #[error("Internal error: failed to construct API URL: {0}")]
    UrlConstruct(String),
    #[error("API request failed: {0}")]
    ApiRequest(String),
    #[error("Failed to parse release JSON: {0}")]
    JsonParse(String),
    #[error("No assets matched pattern '{pattern}'. Available:\n  {available}")]
    NoMatch { pattern: String, available: String },
    #[error("Pattern '{pattern}' matched multiple assets:\n  {matched}")]
    MultipleMatches { pattern: String, matched: String },
    #[error("--output path '{0}' is an existing directory; use --dir instead")]
    OutputIsDir(String),
    #[error("Failed to create directory '{path}': {source}")]
    CreateDir { path: String, #[source] source: std::io::Error },
    #[error("Failed to create file '{path}': {source}")]
    CreateFile { path: String, #[source] source: std::io::Error },
    #[error("Failed to write '{path}': {source}")]
    WriteFile { path: String, #[source] source: std::io::Error },
    #[error("Download failed: {0}")]
    Download(String),
    #[error("Unsupported archive format: '{0}'")]
    UnsupportedFormat(String),
    #[error("Entry '{0}' is a symlink; symlink extraction is not supported")]
    SymlinkEntry(String),
    #[error("Entry '{0}' not found in archive. Top-level entries:\n  {1}")]
    EntryNotFound(String, String),
    #[error("Failed to read archive: {0}")]
    ArchiveRead(String),
    #[error("Failed to extract archive: {0}")]
    ArchiveExtract(String),
}
```

Placed in `src/error.rs` and `pub use`d from each module that needs it.

**Alternative — keep `String` errors:** Simpler but prevents pattern-matching on error kinds and requires every caller to write format strings. Rejected in favour of `thiserror` as per requirements.

### `run()` extraction

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let api_url = to_api_url(&cli.url)?;
    let release = fetch_release(&api_url)?;
    let asset = select_asset(&release.assets, &cli.pattern)?;
    // dispatch to extract_entry / extract_archive / save_to_file
    Ok(())
}
```

This eliminates six repeated `match ... { Err(e) => { eprintln!; exit(1) } }` blocks.

## Risks / Trade-offs

- **`impl Read` lifetime complexity** — `fetch_asset` returns a reader tied to the response lifetime. Callers must not drop the response before finishing the read. This is enforced by the borrow checker; no runtime risk.
- **Larger diff for a pure refactor** — Moving code across files with renamed types makes the diff look large. Tests should serve as the safety net; run them before and after each module split.
- **`error.rs` as a catch-all** — A single error enum covering all modules is simpler now but may need splitting if the codebase grows significantly.

## Open Questions

- None. All decisions are resolved for this scope.
