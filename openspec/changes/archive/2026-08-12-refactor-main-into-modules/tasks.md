## 1. Dependencies

- [x] 1.1 Add `thiserror` to `Cargo.toml`

## 2. Error type

- [x] 2.1 Create `src/error.rs` with the `AppError` enum using `thiserror`

## 3. Module: github

- [x] 3.1 Create `src/github.rs` with `Asset`, `Release` structs
- [x] 3.2 Move `to_api_url()`, `fetch_release()`, `select_asset()` to `src/github.rs`
- [x] 3.3 Add `fetch_asset()` to `src/github.rs` returning `impl Read`
- [x] 3.4 Update all functions in `github.rs` to return `Result<_, AppError>`

## 4. Module: archive

- [x] 4.1 Create `src/archive.rs`
- [x] 4.2 Move `is_extractable()`, `normalize_entry_path()`, `unpack_tar_gz()`, `extract_entry_from_reader()` (renamed to `extract_archive_entry()`) to `src/archive.rs`
- [x] 4.3 Rename `download_asset` → `save_to_file(reader: impl Read, dest: &Path)` in `src/archive.rs`
- [x] 4.4 Rename `extract_asset` → `extract_archive(reader: impl Read, dest_dir: &Path)` in `src/archive.rs`
- [x] 4.5 Update all functions in `archive.rs` to return `Result<_, AppError>`

## 5. Module: output

- [x] 5.1 Create `src/output.rs`
- [x] 5.2 Move `resolve_output_path()` to `src/output.rs` returning `Result<_, AppError>`

## 6. Refactor main

- [x] 6.1 Declare `mod error; mod github; mod archive; mod output;` in `src/main.rs`
- [x] 6.2 Extract `run() -> Result<(), AppError>` from `main()`, using `?` throughout
- [x] 6.3 Update `main()` to call `run()` and handle error once
- [x] 6.4 Update `run()` to call `fetch_asset()` then pass reader to `save_to_file` / `extract_archive` / `extract_archive_entry`
- [x] 6.5 Remove all now-unused code from `src/main.rs`

## 7. Verify

- [x] 7.1 Run `just test` — all tests pass
- [x] 7.2 Run `just lint` — no warnings
- [x] 7.3 Run `just format` — no changes
