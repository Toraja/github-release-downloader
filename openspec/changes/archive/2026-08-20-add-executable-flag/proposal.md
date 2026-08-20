## Why

Release binaries are frequently downloaded (or extracted from a `.tar.gz`) only to require a manual `chmod +x` before they can be run. Adding an `--executable` flag lets the CLI hand back a ready-to-run file in one step, which is the dominant use case for a release-asset downloader.

## What Changes

- Add an `--executable` / `-e` boolean flag to the `download` subcommand that sets the executable bit on the resulting file.
- Permission semantics: OR `0o111` onto the target's current mode (i.e. `chmod a+x` — grant execute to user, group, and other, umask-independent). Existing bits are preserved.
- Target resolution:
  - **Without `--extract`**: the downloaded file.
  - **With `--extract` and `--archive-entry`**: the single extracted entry.
  - **With `--extract` but without `--archive-entry`**: rejected — whole-archive extraction has no single unambiguous target.
- A directory target is treated as user misuse: when `--archive-entry` resolves to a **directory**, extraction still completes and prints its success line, then the CLI emits an error and exits non-zero (no permissions changed). Per ADR 0009, this is only detectable after extraction, so it is a post-extraction error.
- Partial success on `chmod` failure: the download/extraction has already succeeded; the CLI prints the success line, then reports the permission failure and exits non-zero, making clear the asset itself is on disk.
- Short-flag layout: `-e` is newly assigned to `--executable`. Existing flags are unchanged (`-x` `--extract`, `-X` `--archive-entry`), so this is purely additive.
- Platform guard: setting the executable bit is Unix-only (`std::os::unix::fs::PermissionsExt`, covers Linux and macOS). Guard with `#[cfg(unix)]`; the crate still compiles on non-Unix targets, where `--executable` degrades gracefully by failing at runtime with an "not supported on non-Unix platforms" error (not a compile error). Windows is not a supported target (see ADR 0010).

## Capabilities

### New Capabilities
- `executable-permission`: Defines the `--executable` flag, its `a+x` (`0o111`) semantics, target resolution across download / `--extract` / `--archive-entry`, the whole-archive rejection, the directory-target post-extraction error, `chmod`-failure partial-success behaviour, and the Unix-only platform guard.

### Modified Capabilities
<!-- None. Existing extraction and download behaviour is unchanged; --executable only acts on the already-resolved target and adds its own validation rules. -->

## Impact

- `src/main.rs`: new `executable` field on `Download`; new post-parse validation rule (`--executable` + `--extract` without `--archive-entry`) in `try_validate`, alongside the existing `--output` rule; apply-permission call after `save_to_file` / `extract_archive` using the returned `landing`; post-extraction `landing.is_dir()` check for the directory-target error; separate success/permission messaging.
- `src/fs.rs`: `set_executable` helper (Unix-guarded) that ORs `0o111` onto the target's mode, alongside `save_to_file`.
- `src/error.rs`: new `AppError` variant(s) for permission-set failure and executable-on-directory.
- Shell completions regenerate automatically from the clap definition (`shell-completion` capability output changes, no code change).
- No new dependencies; uses `std::os::unix::fs::PermissionsExt`.
- Aligns with ADR 0009 (error vs warning policy) and ADR 0008 (typed `AppError`).
