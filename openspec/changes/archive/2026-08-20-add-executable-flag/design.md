## Context

See `proposal.md` — Why. The `download` flow (`src/main.rs::run`) resolves a `Destination` (from `src/destination.rs`) and calls either `save_to_file` (`src/fs.rs`, plain download) or `extract_archive` (`src/archive.rs`, extract mode), both of which return the produced path as `landing: PathBuf`. Argument rules that clap cannot express declaratively are enforced post-parse in `Download::try_validate`, before any network request. The typed error model is `AppError` (`src/error.rs`, ADR 0008). Error-vs-warning handling follows ADR 0009: user misuse → error; archive-internal noise → warn + continue.

This design covers how `--executable` slots into that flow. See `specs/executable-permission/spec.md` for the behavioral contract.

## Goals / Non-Goals

**Goals:**
- Reuse the existing `landing` return value as the permission target — no new path plumbing.
- Keep the whole-archive rejection in the existing pre-network validation path so no HTTP request is wasted.
- Confine platform-specific code to one small, guarded helper.

**Non-Goals:**
- Recursive permission changes on directory targets (directory is a hard error).
- Replicating shell `chmod +x` umask sensitivity — the spec fixes semantics to unconditional `mode |= 0o111`.
- Windows support.

## Decisions

### D1: Target is the `landing` path returned by the write/extract functions
Both `save_to_file` and `extract_archive` already return the final path. The permission step consumes that value in `run`, after the success message is printed. No signature changes to those functions for the download and single-entry paths.

*Alternative — thread an `executable` flag into `save_to_file`/`extract_archive`:* rejected. It couples permission policy into the write layer and duplicates it across two call sites; keeping it in `run` centralizes the policy and keeps ADR 0009's "print success line, then act" ordering explicit and visible.

### D2: Whole-archive rejection lives in `try_validate` (pre-network)
`--executable && extract && archive_entry.is_none()` is a conditional constraint clap cannot express, exactly like the existing `--output` rule. Add it next to that rule in `Download::try_validate`, returning a clap `ArgumentConflict` error. This preserves the invariant that misuse detectable from arguments alone fails before any HTTP request.

*Alternative — clap `requires`/`conflicts_with`:* rejected; the rule is conditional on `archive_entry` being absent, which clap's declarative attributes can't express (the codebase already documents this for `--output`).

### D3: Directory detection is post-extraction, using the returned `landing`
Directory-ness of an `--archive-entry` target is only knowable after streaming the archive. After `extract_archive` returns `landing`, if `--executable` is set, check `landing.is_dir()`. If it is a directory: the "Extracted to:" line has already been printed, so emit a new `AppError` (executable-on-directory) and exit non-zero without changing permissions. This matches the spec's post-extraction error and ADR 0009's misuse-after-work case.

*Alternative — detect during `extract_archive_entry` (it already distinguishes file vs directory match):* rejected for now. It would push permission policy back into the extraction layer (contra D1) and complicate its return type; a single `is_dir()` check in `run` is sufficient and keeps the concern in one place.

### D4: Permission helper is a single `#[cfg(unix)]` function
Add a helper (e.g. `set_executable(path) -> Result<(), AppError>`) in `src/fs.rs`, alongside `save_to_file` (both are filesystem side-effects on a resolved path):
- `#[cfg(unix)]` body: read current mode via `fs::metadata(path)?.permissions()`, `set_mode(mode | 0o111)`, `fs::set_permissions(path, perms)?`. Wrap I/O errors in a new `AppError` variant.
- `#[cfg(not(unix))]` body: return an `AppError` (a runtime `io::ErrorKind::Unsupported` error) so the crate still compiles on non-Unix hosts and `--executable` degrades gracefully at runtime, consistent with "Windows not supported" (ADR 0010). This is deliberately not a `compile_error!`: the build must succeed on non-Unix so contributors on those hosts are not blocked.

`std::os::unix::fs::PermissionsExt` covers Linux and macOS, our only shipped targets.

*Alternative — no cfg guard:* rejected; an unguarded `use std::os::unix::...` breaks `cargo build`/`just lint` for any contributor on a non-Unix host even though we don't ship those targets.

### D5: New `AppError` variants
Add to `src/error.rs`:
- `SetPermissions { path, source: io::Error }` — chmod failure (partial success; message must convey the asset is already on disk).
- `ExecutableTargetIsDir(String)` — `--executable` on a directory entry.

Follows ADR 0008 (typed enum, message defined once on the variant).

### D6: Message ordering
Success line (`Downloaded:` / `Extracted to:`) is printed first, unconditionally, before the permission step runs. Any permission error is emitted afterward. This guarantees that on `SetPermissions` or `ExecutableTargetIsDir`, the user always learns the download/extraction itself completed.

## Risks / Trade-offs

- **Directory error leaves files on disk with a non-zero exit** → Intentional per ADR 0009 and the spec; the ordering in D6 plus a message that names the directory makes the state clear and actionable.
- **`chmod` failure after a successful write** → Same partial-success handling as existing write errors; D6 ordering ensures the user doesn't re-download.
- **`0o111` also grants group/other execute, which some may find broad** → Deliberate, umask-independent choice fixed by the spec for predictability; documented on the flag and in the spec.
- **cfg-guarded code paths are easy to leave untested on the non-Unix branch** → Acceptable: only Unix targets are shipped and tested; the non-Unix branch exists solely to fail clearly at runtime while keeping the crate buildable on non-Unix.
