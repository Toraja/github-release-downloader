## Context

`src/archive.rs` currently has a single extraction pipeline built around `flate2` + `tar`: `extract_archive(reader, entry, dest)` takes any `Read` and streams the response body straight into a `GzDecoder`/`tar::Archive`, for both whole-archive (`unpack_tar_gz`) and single-entry (`extract_archive_entry`) modes. Format detection is `is_extractable()` (`.tar.gz`/`.tgz` suffix check), and the caller in `main.rs` guards `--extract`/`--archive-entry` with it before downloading.

The zip format stores its central directory at the end of the file, so readers need `Seek` — the existing "stream the HTTP response body" approach cannot work for `.zip`. See proposal.md for motivation (issue #15, prerequisite for #8) and the delta specs for the behavioural contract.

## Goals / Non-Goals

**Goals:**
- `.zip` assets work with `--extract` (whole archive) and `--extract --archive-entry` (file entry, directory entry, symlink rules, entry-not-found listing, parent-dir creation, merge-into-existing).
- Keep the tar.gz path untouched behaviourally; share the entry-matching logic where practical.
- Archive bytes are never written to disk for either format.

**Non-Goals:**
- Other archive formats (`.tar.bz2`, `.tar.xz`, `.7z`, …).
- Changing download (non-extract) behaviour — `.zip` assets already download fine.
- Preserving zip entry permissions/attributes beyond what the tar path already does (the tar path relies on `tar::Entry::unpack` defaults; zip will similarly use library defaults).

## Decisions

### 1. Buffer the response body in memory for zip, keep streaming for tar.gz

`extract_archive` keeps its `impl Read` signature. `is_extractable` is replaced by a format classifier, e.g. `enum ArchiveFormat { TarGz, Zip }` + `fn detect_format(asset_name) -> Option<ArchiveFormat>`, which selects the `Extractor` implementation (Decision 3). The `TarGz` impl streams the reader as today; the `Zip` impl drains the reader into an owned `Vec<u8>` + `Cursor` at construction, satisfying `zip::ZipArchive: Read + Seek` — so the buffering difference is confined to the factory and invisible to callers.

- *Alternative: stream zip via `zip::read::read_zipfile_from_stream`.* Rejected. This API reads local file headers sequentially from a non-seekable reader, but the stream carries only local headers — the central directory (which holds the authoritative metadata) is skipped. Concretely, per the crate docs, fields are missing when reading this way, including `external_attributes`, so `unix_mode()` returns `None` for every entry. That breaks our symlink policy: symlink detection on zip relies on the unix mode high bits (`S_IFLNK`), so a streamed symlink entry would be indistinguishable from a regular file and could be unpacked as one — silently defeating the "specified entry is a symlink → error" and "child symlinks skipped" requirements. It is also a poor fit for the entry-not-found requirement: the top-level entry listing would have to be accumulated during the single forward pass (fine), but a mid-stream I/O or decompression failure would leave us unable to distinguish "entry genuinely absent" from "stream broke before we reached it", and there is no way to re-inspect entries without re-downloading. The docs themselves advise: "If possible, use the ZipArchive functions." Since we must buffer for correctness anyway, `ZipArchive` + `Cursor<Vec<u8>>` is strictly simpler: random access by index, full central-directory metadata (`unix_mode()`, reliable names via `enclosed_name()`), and trivially repeatable iteration for the not-found listing.
- *Alternative: spool to a temp file.* Rejected: violates the "archive not saved to disk" requirement for no real benefit; release assets are typically a few MB to a few tens of MB.

### 2. Use the `zip` crate (current stable, 8.x line)

The `zip` crate (maintained at `github.com/zip-rs/zip2`) is the de-facto standard; latest stable is 8.6.0 at the time of writing. Use `zip = "8"` with `default-features = false` and only the features we need (`deflate` covers the common case; optionally `deflate64` for Windows-produced archives) — the default feature set pulls in `aes-crypto`, `bzip2`, `xz`, `zstd`, etc., which we don't need. Note the MSRV is 1.88 and the project's `edition = "2024"` already requires a recent toolchain. Avoid `9.0.0-pre3` (pre-release).

- *Alternative: `zip-extensions`.* Rejected: a thin convenience wrapper; we need entry-level control anyway.

### 3. Define an `Extractor` trait with one implementation per format

Introduce a trait in `src/archive.rs` capturing the two extraction operations, with a factory that constructs the right implementation from the detected format:

```rust
trait Extractor {
    /// Unpack the entire archive into `dest_dir`.
    fn unpack_all(&mut self, dest_dir: &Path) -> Result<(), AppError>;

    /// Extract the file or directory entry `entry` to `dest`,
    /// applying the shared matching/symlink/not-found rules.
    fn extract_entry(&mut self, entry: &str, dest: &Path) -> Result<(), AppError>;
}

fn extractor_for(format: ArchiveFormat, reader: impl Read + 'static)
    -> Result<Box<dyn Extractor>, AppError>
```

- `TarGzExtractor` wraps the `Read` lazily and streams when used (preserves the current early-exit-on-exact-match behaviour).
- `ZipExtractor` drains the reader into an owned `Vec<u8>`/`Cursor` at construction (per Decision 1). Because `ZipArchive<'a, R>` borrows its reader, the struct owns the buffer and builds a fresh `ZipArchive` view inside each method call — plain safe code, no self-reference.

Rationale:

- **Format differences move to construction and per-entry I/O, where they belong.** `extract_archive` collapses to: detect format → `extractor_for(...)` → resolve destination → `unpack_all`/`extract_entry`. No `if zip { buffer }` branches leaking into the shared flow.
- **Shared policy stays shared.** Path normalisation, destination resolution (`Destination::Into`/`Exact` → landing path), and the matching rules (exact → file, prefix → directory, symlink error/skip, top-level listing on not-found) remain common helpers both impls call; the trait bounds only what *must* differ (entry iteration and per-entry unpack).
- **Issue #8 falls out naturally.** The next format is a new impl + one factory arm, not a third copy of the matching logic — cheap foresight now rather than a refactor later.
- **Clean test seam.** The existing test matrix (`make_tar_gz_with_entries` → `extract_archive(...)`) mirrors as per-format helpers (`make_zip_with_entries`) exercised through the same trait.

Guardrails against over-abstraction:

- **Do not abstract entry iteration.** The trait exposes whole operations (`unpack_all`, `extract_entry`), each impl iterating internally its own way — tar streams and short-circuits on exact match, zip uses random access by index with `enclosed_name()` (which also sanitises path traversal) and `by_name`. An iterator-style abstraction ("yield entry metadata") would sacrifice both advantages.
- Symlink detection: tar uses `entry_type().is_symlink()`; zip uses `unix_mode()` high bits (`S_IFLNK`). Zip files from Windows often carry no unix mode — such entries are treated as regular files, which is the desired behaviour.

- *Alternative: free functions with shared helpers (the status quo extended).* Rejected: the format branch would live inside `extract_archive` and every future format adds another inline path; the trait makes the contract explicit and the buffering difference invisible to callers.

### 4. Update format-guard messages and help text

`AppError::UnsupportedFormat` (error.rs) and the `--extract`/`--archive-entry` doc comments in `main.rs` are updated to list `.tar.gz`, `.tgz`, and `.zip`. Validation stays pre-download, in `main.rs`, keyed off the asset filename — unchanged in structure.

## Risks / Trade-offs

- [Memory usage: whole zip buffered in RAM] → Acceptable for typical release assets; note the limitation in the error message path is unnecessary, but document the trade-off here. Revisit if issue #8 (large archives) demands otherwise.
- [Two `Extractor` impls could drift on matching rules / symlink policy] → The trait bounds only iteration and per-entry unpack; normalisation, matching, and symlink policy live in shared helpers both impls must call. Mirror the existing test matrix for zip (`make_zip_with_entries` helper analogous to `make_tar_gz_with_entries`) through the same trait so behavioural drift shows up as test failures.
- [Zip entry names may use `\` separators or lack unix modes (Windows-produced archives)] → Use `enclosed_name()` for sanitisation and normalise separators when matching; covered by tests.
- [Crate version churn in `zip` API across major versions] → Pin one stable major (`zip = "8"`) and stick to the stable `ZipArchive` API surface only (avoid the `unstable` module, which may change in patch releases).

## Migration Plan

Pure additive feature; no migration. Rollback = revert the commit; no persisted state involved.
