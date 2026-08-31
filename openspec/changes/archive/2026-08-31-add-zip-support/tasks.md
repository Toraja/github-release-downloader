## 1. Dependency & format detection

- [x] 1.1 Add `zip = { version = "8", default-features = false, features = ["deflate", "deflate64"] }` to Cargo.toml and verify `cargo build` succeeds
- [x] 1.2 In src/archive.rs, replace `is_extractable` with `enum ArchiveFormat { TarGz, Zip }` and `fn detect_format(asset_name: &str) -> Option<ArchiveFormat>` (recognising `.tar.gz`, `.tgz`, `.zip`), update callers in src/main.rs, and verify `just test` passes with updated detection unit tests (`.zip` now accepted, `.tar.bz2`/no-extension still rejected)

## 2. Extractor trait refactor (no behaviour change)

- [x] 2.1 Define the `Extractor` trait (`unpack_all`, `extract_entry`) plus `extractor_for(format, reader) -> Result<Box<dyn Extractor>, AppError>` in src/archive.rs per design.md Decision 3, and verify `cargo build` succeeds
- [x] 2.2 Move the existing tar.gz whole-archive logic (`unpack_tar_gz`) and single-entry logic (`extract_archive_entry`) into `TarGzExtractor`, keeping path normalisation, destination resolution, and matching/symlink/not-found rules in shared helpers, and verify all existing tests pass unchanged with `just test`

## 3. Zip extractor

- [x] 3.1 Implement `ZipExtractor::new` draining the reader into an owned `Vec<u8>`/`Cursor` (building a fresh `ZipArchive` view per method call) and implement `unpack_all`, verifying with a unit test that a `make_zip_with_entries` fixture extracts fully into a `--dir`-style destination
- [x] 3.2 Implement `ZipExtractor::extract_entry` reusing the shared matching helpers — exact file match, directory prefix match, `enclosed_name()` path-traversal sanitisation, `\` separator normalisation, symlink detection via `unix_mode()` high bits (direct symlink → `SymlinkEntry` error; child symlink → warn + skip), parent-dir creation, merge-into-existing, and entry-not-found listing top-level entries — verifying each behaviour with a unit test mirroring the tar.gz test matrix
- [x] 3.3 Verify no `.zip` file is left on disk after entry extraction by extending the not-found/no-archive-file tests to zip

## 4. Messages & help text

- [x] 4.1 Update `AppError::UnsupportedFormat` in src/error.rs to list `.tar.gz`, `.tgz`, and `.zip`, and verify the error-path test (or a manual `ghrls --extract` run against an unsupported asset name) prints the new message and exits non-zero before any download
- [x] 4.2 Update the `--extract` and `--archive-entry` doc comments in src/main.rs to mention `.zip` support and verify `ghrls --help` output reflects it

## 5. Final verification

- [x] 5.1 Run `just test`, `just lint`, and `just format`; all pass cleanly
- [x] 5.2 Manually verify end-to-end against a real GitHub release with a `.zip` asset: `ghrls --extract` (whole archive) and `ghrls --extract --archive-entry <path>` both succeed and print `Extracted to: <dest>`
