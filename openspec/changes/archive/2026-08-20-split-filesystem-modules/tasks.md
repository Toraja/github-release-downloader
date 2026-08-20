## 1. Create `destination.rs`

- [x] 1.1 Create `src/destination.rs` and move the `Destination` enum and its `impl` (`resolve`) out of `src/archive.rs`, keeping `pub` visibility unchanged
- [x] 1.2 Move the `Destination::resolve` unit tests (the `test_destination_resolve_*` tests) into `destination.rs`'s `#[cfg(test)] mod tests`
- [x] 1.3 Add any needed imports in `destination.rs` (`std::path::{Path, PathBuf}`, `crate::error::AppError`)

## 2. Create `fs.rs`

- [x] 2.1 Create `src/fs.rs` and move `save_to_file` out of `src/archive.rs`, keeping `pub` visibility unchanged
- [x] 2.2 Add imports in `fs.rs` (`std::fs`, `std::fs::File`, `std::io`, `std::path::PathBuf`, `crate::destination::Destination`, `crate::error::AppError`)
- [x] 2.3 Move the `save_to_file` unit tests (the `test_save_to_file_*` tests) into `fs.rs`'s `#[cfg(test)] mod tests`

## 3. Reduce `archive.rs` to extraction only

- [x] 3.1 Remove `Destination`/`resolve` and `save_to_file` (now moved) from `src/archive.rs`, leaving `is_extractable`, `extract_archive`, `unpack_tar_gz`, `extract_archive_entry`, `normalize_entry_path`
- [x] 3.2 Add `use crate::destination::Destination;` to `archive.rs` and prune now-unused imports
- [x] 3.3 Confirm the extraction unit tests remain in `archive.rs` (do not move them)

## 4. Wire up `main.rs`

- [x] 4.1 Add `mod destination;` and `mod fs;` alongside the existing module declarations in `src/main.rs`
- [x] 4.2 Update `use` statements: import `Destination` from `crate::destination`, `save_to_file` from `crate::fs`, and extraction items (`extract_archive`, `is_extractable`) from `crate::archive`

## 5. Verify no behavior change

- [x] 5.1 Run `just test` and confirm all tests pass (relocated tests included)
- [x] 5.2 Run `just lint` and confirm no warnings/errors
- [x] 5.3 Run `just format` and confirm formatting is clean
