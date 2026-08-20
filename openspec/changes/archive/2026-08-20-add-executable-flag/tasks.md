## 1. Error variants (`src/error.rs`)

- [x] 1.1 Add `AppError::SetPermissions { path: String, source: io::Error }` with a message conveying that the asset was written but its executable bit could not be set
- [x] 1.2 Add `AppError::ExecutableTargetIsDir(String)` with a message stating `--executable` requires a file target (naming the directory)

## 2. Permission helper (`src/fs.rs`)

- [x] 2.1 Add `#[cfg(unix)] pub fn set_executable(path: &Path) -> Result<(), AppError>` that reads the current mode, ORs `0o111`, and applies it via `fs::set_permissions`, wrapping I/O errors in `AppError::SetPermissions`
- [x] 2.2 Add a `#[cfg(not(unix))]` variant of `set_executable` that fails clearly (Windows unsupported), so the crate still compiles on non-Unix hosts
- [x] 2.3 Add unit tests for `set_executable`: `0o644` → `0o755`, `0o600` → `0o711`, `0o755` unchanged (Unix-only, gated with `#[cfg(unix)]`)

## 3. CLI flag + validation (`src/main.rs`)

- [x] 3.1 Add the `#[arg(short = 'e', long)] executable: bool` field to `Download`, with help text describing `a+x` semantics and Unix-only support
- [x] 3.2 In `Download::try_validate`, add the rule: `executable && extract && archive_entry.is_none()` → clap `ArgumentConflict` error directing the user to `--archive-entry` (pre-network, alongside the existing `--output` rule)
- [x] 3.3 Add a parse-level unit test asserting `--extract --executable` (no `--archive-entry`) yields `ErrorKind::ArgumentConflict` via `try_validate`
- [x] 3.4 Add a parse-level unit test asserting `--extract --archive-entry bin/tool --executable` parses and validates successfully

## 4. Wire permission step into `run` (`src/main.rs`)

- [x] 4.1 In the extract branch: after printing `Extracted to:`, if `executable` is set, error with `ExecutableTargetIsDir` when `landing.is_dir()`, otherwise call `fs::set_executable(&landing)?`
- [x] 4.2 In the plain-download branch: after printing `Downloaded:`, if `executable` is set, call `fs::set_executable(&landing)?`
- [x] 4.3 Verify message ordering (D6): the success line is always printed before any permission error is emitted

## 5. Verify

- [x] 5.1 Run `just test` and confirm all tests pass
- [x] 5.2 Run `just lint` and confirm no warnings/errors
- [x] 5.3 Run `just format` and confirm formatting is clean
