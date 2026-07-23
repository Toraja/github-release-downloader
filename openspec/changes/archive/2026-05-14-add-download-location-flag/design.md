## Context

The tool currently hard-codes the save location to the current working directory. When users script this tool or invoke it from CI environments, they often need assets saved to a specific path without first changing directories. This is a small, self-contained CLI change confined to the argument parsing and file-write layers.

## Goals / Non-Goals

**Goals:**
- Add an optional `--dir` / `-D` flag accepting a destination directory path
- Add an optional `--output` / `-O` flag accepting an exact file path (enables rename)
- `--dir` and `--output` are mutually exclusive, enforced by clap
- Automatically create parent directories if they do not exist
- Preserve current default behaviour when neither flag is omitted

**Non-Goals:**
- Changing any other aspect of the download pipeline (URL resolution, auth, asset matching)

## Decisions

### Option flag vs positional argument
**Decision**: Use optional named flags rather than a third positional argument.

Alternatives considered:
- Third positional arg: positional args are harder to discover and easy to mis-order; named flags are self-documenting and conventional for output paths in Unix tools.

### Two separate flags: `--dir` and `--output`
**Decision**: Use `--dir` / `-D` for destination directory and `--output` / `-O` for exact file path, mutually exclusive via clap's `conflicts_with`. This follows the same convention as `gh release download`.

Alternatives considered:
- Single `--output` flag with runtime dir-vs-file heuristic (trailing slash / existing dir): discarded because it interacts poorly with a future tarball-extract feature, where the output is always a directory regardless of path shape. Separate flags keep the semantics unambiguous and make the flags naturally composable with future features via `conflicts_with`.

### Auto-create parent directories
**Decision**: Call `fs::create_dir_all` on the target directory before writing, so the user can specify a not-yet-existing path and have it created automatically.

Rationale: Consistent with `wget -P`, `curl --output-dir`, and `gh release download -D`. Reduces friction in scripted/CI usage where directory setup may not be a separate step.

## Risks / Trade-offs

- [Ambiguity between file and directory path] → Eliminated by using two separate flags with clear names.
- [Silent creation of unexpected directories due to typos] → Acceptable trade-off given the explicit opt-in nature of the flags; users who omit both flags are unaffected.
- [Breaking CI scripts that rely on current directory output] → No risk; both flags are optional, default behaviour unchanged.
