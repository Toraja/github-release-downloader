# Tasks

## 1. Zip extraction in archive.rs

- [ ] 1.1 Add `zip` crate dependency with `deflate` feature to Cargo.toml and verify `cargo check` succeeds.
- [ ] 1.2 Introduce format routing in `src/archive.rs`: detect zip by extension (`.zip`, `.vsix`); keep existing `.tar.gz`/`.tgz` streaming. Verify existing `is_extractable` tests still pass.
- [ ] 1.3 Implement in-memory zip path: buffer archive body into `Vec<u8>`, open with `zip::ZipArchive::new(Cursor<Vec<u8>>)`; whole-archive extraction writes all entries. Verify new unit test extracting a synthetic zip buffer created with `zip::ZipWriter` matches expectations.
- [ ] 1.4 Extend single-entry extraction for zip buffers: prefix-match directory, strip the directory wrapper at landing (i.e. `extension/` prefix removed and contents placed directly at `dest`). Verify new test that a synthetic zip's `extension/<name>` entry lands at `dest/<name>` without the wrapper.
- [ ] 1.5 Ensure zip error handling maps to `AppError::Archive*`. Verify error tests for malformed zip and missing entry.
- [ ] 1.6 Refactor `is_extractable` into a small format enum (e.g., `SupportedFormat::Tar | SupportedFormat::Zip`) if routing needs it. Verify `download --extract` still accepts tar.gz and rejects unknowns; vsix accepts `.vsix`.

## 2. Vsix subcommand in main.rs

- [ ] 2.1 Add `Command::Vsix(Vsix)` variant with args `--dir`, `-D` only (no `--output`). Reject `--output` with `clap` conflict error. Verify parser test in main.rs.
- [ ] 2.2 Validating `.vsix` extension on matched asset before download. Return new `AppError::NotVsixExtension`. Verify parser/validation test.
- [ ] 2.3 For vsix run path: fetch asset, resolve destination to `Destination::Exact(--dir or cwd)`, extract via zip path with entry `"extension"` with wrapper-stripped landing, then print `Extracted to: <dest>`.
- [ ] 2.4 Ensure meta-files like `[Content_Types].xml` and `extension.vsixmanifest` outside `extension/` are skipped automatically by the prefix mode; add test confirming only `extension/` contents land.

## 3. Documentation and checks

- [ ] 3.1 Add `README.md` section describing `ghrls vsix <URL> <PATTERN> [-D <DIR>]`. Verify `just lint` and markdown validations pass (`just format`).
- [ ] 3.2 Update `CHANGELOG.md` under `[Unreleased]` → Added. Verify `just lint` passes.
- [ ] 3.3 Run full `just test` and `just lint` and `just format` and fix any failures.

## 4. Validation

- [ ] 4.1 Run `openspec validate add-vsix-support --strict` and ensure still valid.
- [ ] 4.2 Run a manual end-to-end: `ghrls vsix https://github.com/vadimcn/codelldb 'codelldb-linux-x64\.vsix' --dir /tmp/vsix-test` and confirm `/tmp/vsix-test/package.json` exists (extension wrapper stripped).