## Context

The tool currently supports two extraction modes: full archive extraction (`--extract`) and direct file download (`--output`/`--dir`). These are kept mutually exclusive because streaming an archive to a named output path makes no semantic sense.

A common real-world need is extracting a single binary or config directory from a tarball release (e.g. extracting only `bin/mytool` from a multi-file archive) and optionally renaming it at the destination. There is no clean way to achieve this today without downloading the archive first and extracting manually.

The existing flag surface is:
- `-x` / `--extract`: bool, conflicts with `--output`
- `-O` / `--output`: `Option<PathBuf>`, conflicts with `--dir`
- `-D` / `--dir`: `Option<PathBuf>`, conflicts with `--output`

## Goals / Non-Goals

**Goals:**
- Add a flag (`--extract-entry`) that selects a single file or directory entry within a `.tar.gz`/`.tgz` archive to extract.
- Allow the extracted entry to be renamed at the destination via `--output`.
- Keep `--dir` usable as the destination parent directory (same semantics as today).
- Keep the existing `--extract` behaviour completely unchanged.

**Non-Goals:**
- Extracting multiple entries in a single invocation.
- Supporting archive formats beyond `.tar.gz`/`.tgz`.
- Allowing `--extract-entry` alongside `--extract` (mutually exclusive by design).

## Decisions

### Flag name and type: `--extract-entry <PATH>`

`--extract-entry` takes a `String` value representing the path of a file or directory within the archive (e.g. `bin/mytool` or `share/`). A `String` is preferred over `PathBuf` because archive entry paths are not native filesystem paths and normalisation (e.g. stripping leading `./`) is done explicitly in code.

Short flag: `-X`.

**Constraints declared via clap:**
```
conflicts_with = "extract"
```
`--output` is already conflict-free with respect to `--extract-entry` — its only existing `conflicts_with` is `"dir"`. No changes to `--output` or `--dir` clap annotations are required.

**Alternative considered:** A combined `--extract-entry <ARCHIVE_PATH>:<DEST>` syntax to fold renaming into one flag. Rejected — it is non-standard, harder to parse, and `--output` already handles renaming cleanly.

### Entry matching semantics

Archive entry paths are normalised by stripping a leading `./` (common in tarballs built with `tar -czf`). The user-supplied `--extract-entry` value is also normalised the same way, and a trailing `/` is stripped.

Matching rules:
- **File entry**: an archive entry whose normalised path equals the normalised `--extract-entry` value.
- **Directory entry**: one or more archive entries whose normalised paths start with `<normalised-value>/`.

If no matching entries are found, the command exits with an error listing available top-level entries to help the user.

**Alternative considered:** Glob/regex matching inside the archive. Rejected — the proposal explicitly limits extraction to a single entry, and fuzzy matching makes the output non-deterministic.

### Destination resolution for `--extract-entry`

| Combination | File entry destination | Directory entry destination |
|---|---|---|
| `--output <path>` | `<path>` (exact) | `<path>/` (created if absent) |
| `--dir <d>` | `<d>/<basename(entry)>` | `<d>/<basename(entry)>/` |
| (neither) | `./<basename(entry)>` | `./<basename(entry)>/` |

The `resolve_output_path` helper is not reused here because its `is_dir()` guard (which rejects an existing directory as an `--output` path) is inappropriate when the target is a directory entry. A new `resolve_entry_dest` function handles this case.

### New extraction function: `extract_entry`

A new function `extract_entry(asset, entry, dest_root)` is added alongside the existing `extract_asset`. It:
1. Streams the archive (same HTTP fetch as `extract_asset`).
2. Iterates tar entries, normalises their paths.
3. For file entries matching exactly: writes the single file to `dest_root`.
4. For directory entries matching by prefix: strips the matched prefix, recreates relative paths under `dest_root`.
5. Creates parent directories as needed.

`extract_asset` (full extraction) is untouched.

### Format validation

Reuses the existing `is_extractable` check — if `--extract-entry` is passed and the matched asset is not `.tar.gz`/`.tgz`, the command exits with an error identical in style to the existing `--extract` guard.

## Risks / Trade-offs

- **Archive path opacity**: The user must know the exact internal path of the entry they want. No listing command is provided (out of scope). Mitigation: on entry-not-found error, print available top-level entries to guide the user.
- **Directory entry + `--output` pointing to an existing file**: attempting to create a directory at a path occupied by a file will fail at `fs::create_dir_all`. Mitigation: runtime error with a clear message.
- **Symlinks in tar entries**: Two distinct cases. (1) The specified entry itself is a symlink → exit with error; symlink resolution is not supported. (2) Symlinks appear as children within a directory entry → skip each one and print a warning to stderr, then continue extracting regular files.
