## 1. Flag surface (src/main.rs)

- [x] 1.1 Rename the `extract_entry` field and `--extract-entry` flag to `archive_entry` / `--archive-entry`, keeping short flag `-X`; update its doc comment to describe it as a qualifier of `--extract`.
- [x] 1.2 Replace `conflicts_with = "extract"` on the entry flag with `requires = "extract"`.
- [x] 1.3 Remove `conflicts_with = "output"` from the `--extract` flag.
- [x] 1.4 Add post-parse validation that rejects `--extract` + `--output` when `--archive-entry` is absent, emitting a `clap::error::ErrorKind::ArgumentConflict` error via `Args::command().error(..)` with a message directing the user to `--dir`.

## 2. Unified extraction (src/archive.rs)

- [x] 2.1 Add a `Destination` enum with `Into(PathBuf)` (from `--dir`/default) and `Exact(PathBuf)` (from `--output`).
- [x] 2.2 Introduce a single public `extract_archive(reader, entry: Option<&str>, dest: Destination)` that dispatches: `None` → existing whole-archive unpack; `Some(e)` → existing entry loop.
- [x] 2.3 Inside the unified function, resolve the landing path from `Destination` + `entry` (Into → dir-join-basename for entries / dir-as-is for whole archive; Exact → verbatim), preserving current `--dir`/`--output` behaviour.
- [x] 2.4 Handle the impossible `(entry = None, Destination::Exact)` cell defensively (explicit error or documented `unreachable!` tied to the task 1.4 guard).
- [x] 2.5 Fold the old `extract_archive_entry` body into the unified path and remove the now-redundant public function(s), keeping `unpack_tar_gz` for whole-archive unpacking.

## 3. Dispatch (src/main.rs run())

- [x] 3.1 Collapse the `--extract-entry` and `--extract` branches into one: build `entry = archive_entry.as_deref()` and a `Destination` from `--dir`/`--output`, then call `extract_archive` once.
- [x] 3.2 Keep the plain-download branch unchanged; ensure the `Extracted to: <dest>` message uses the resolved destination for both whole-archive and single-entry cases.

## 4. Tests

- [x] 4.1 Update the existing clap-conflict tests in `main.rs`: rename to `--archive-entry`, replace the extract/entry mutual-exclusion test with a `--archive-entry` requires `--extract` test.
- [x] 4.2 Add a test that `--extract --output` (no `--archive-entry`) is rejected as an argument conflict.
- [x] 4.3 Add a test that `--extract --archive-entry ENTRY --output PATH` parses successfully.
- [x] 4.4 Update `archive.rs` unit tests to call the unified `extract_archive` with `Option<entry>` and the `Destination` enum, covering whole-archive, file-entry (default/`--dir`/`--output`), and directory-entry (default/`--dir`/`--output`) cases without behavioural change.

## 5. Docs & verification

- [x] 5.1 Update `README.md` and flag help text for the renamed flag and the new combination rules (`--extract` + `--archive-entry` [+ `--output`]).
- [x] 5.2 Run `just test`, `just lint`, and `just format`; fix any failures.
