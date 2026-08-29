## Context

See proposal.md — Why. The correction from proposal review: the vsix subcommand is a thin wrapper over the existing extraction machinery; it only validates `.vsix` and hard-codes extraction of the archive entry `extension/`, landing its contents directly at the destination. Zip support inside archive.rs is the genuinely new machinery, and is needed only because vsix happens to be a zip.

The CLI today accepts tar.gz/tgz via `--extract` (streaming) and rejects anything else up-front (`UnsupportedFormat`). Extraction lives in `src/archive.rs`; subcommand wiring lives in `src/main.rs`. The `Destination` enum has two variants — `Into(dir)` (for whole-archive) and `Exact(path)` (for single-entry `--output`) — chosen in `Destination::resolve`.

## Goals / Non-Goals

**Goals:**
- archive.rs gains zip extraction with format routing (tar.gz streamed; zip buffered in-memory).
- `vsix` subcommand exists with `.vsix` validation and hard-coded `extension/` entry extraction.
- Landing semantics for vsix: `--dir <D>` (or cwd default) means "put the contents of `extension/` directly into `<D>`", with `Destination::Exact(<D>)` resolution. `extension/` itself is stripped as a path component.

**Non-Goals:**
- No generic zip support in `download --extract` beyond what vsix requires (only extension format validation differs; `download --extract` does not change its default accept list).
- No `--output` on the vsix subcommand (destination is always a directory landing; a renameable file path is meaningless here).
- No nested-extension discovery (vsix layout is trusted; only root-level `extension/` is extracted).

## Decisions

### Zip lives in archive.rs with minimal vsix-specific help
vsix-specific `.vsix` check and entry `extension/` live in main.rs; archive.rs adds only a zip extractor keyed on a new `AssetFormat::Zip` enum / `is_extractable_zip(name)` refactor.

**Alternatives considered and rejected:**
- Putting `.vsix` guard in archive.rs — ties a vsix-tuned restriction into generic extraction.
- One extractor function adding a mode param — muddies a generic API; harder to test.

### In-memory zip buffering
Zip needs `Read + Seek`; archive.rs buffers the full archive body (`Vec<u8>` → `ZipArchive::new(Cursor<_>)`). codelldb's vsix is ~50MB; memory pressure on modern machines is a non-issue for this tool.

**Alternatives considered and rejected:**
- Writing temp file to disk — violates "never save archive to disk" ethos held by the existing `--extract` path.

### Destination semantics for vsix
Use `Destination::Exact(path-from-flag-or-cwd)` — not `Into` — because we want `extension`-wrapper stripping. `--dir` name remains; the flag's meaning is "directory into which extension contents land directly." Alternative considered and rejected: `Into(dir)` with post-strip step, which requires `./` path math with filesystem rename — brittle and adds fs complexity.

### New error variants
`AppError::NotVsixExtension(name)` and `AssetFormat::Zip` recognition; differentiate vsix validation from unsupported format by which subcommand raised it. Errors stay thiserror-derived as in the current style.

### Hard-coded `extension` entry
Fixed literal `"extension"` in main.rs for clarity. Resist param injection (e.g., adding `--archive-entry` to `vsix`) — the subcommand's purpose is to be opinionated; alternative flexibility defuses it.

## Risks / Trade-offs

- [Risk: In-memory zip buffering OOMs with very large extensions.] → Mitigation: buffer in one `Vec<u8>`; 50MB is the worst seen; safe within current CLI footprint. `ureq` body to reader to `Cursor` is the simplest path.
- [Risk: Malformed vsix lacks `extension/` entry.] → Mitigation: `EntryNotFound` error with zip-toplevel list, mirroring today's tar behavior.
- [Risk: vsix assets like [Content_Types].xml present at root.] → Mitigation: skipped silently per spec; current `extract_archive_entry` in directory prefix mode automatically covers non-matching root entries when prefix `extension/` is used — no code change needed beyond format routing.
- [Risk: Tar.gz extraction behavior changes due to format-routing refactor.] → Mitigation: keep tar matches unchanged (same `.tar.gz`/`.tgz` accept set). Route selection lives behind a small enum to ensure exhaustive cover.

## Migration Plan

No migration. New subcommand, no breaking download semantics changes.
