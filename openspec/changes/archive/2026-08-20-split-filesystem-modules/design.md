## Context

See `proposal.md` — Why. Current module layout (`src/`): `main.rs` (CLI + `run` orchestration), `archive.rs` (`Destination`, `save_to_file`, and tar.gz extraction), `github.rs` (network), `error.rs` (`AppError`). `Destination` is shared vocabulary consumed by both `save_to_file` and `extract_archive`; `save_to_file` is a plain filesystem write unrelated to archives. This is a mechanical, behavior-preserving reorganization.

## Goals / Non-Goals

**Goals:**
- Give each module a single clear responsibility so filesystem side-effects have an obvious home.
- Keep the change a pure move: identical public behavior, identical error types, identical tests (relocated, not rewritten).

**Non-Goals:**
- Any behavior, signature, or error-message change.
- Adding `set_executable` (belongs to `add-executable-flag`); this change only establishes where it will live.
- Touching `github.rs`, `error.rs`, dependencies, or the CLI surface.

## Decisions

### D1: `Destination` goes in a neutral `destination.rs`, not with the writer
`Destination` + `resolve` move to `src/destination.rs`. It is shared vocabulary used by both the write path (`fs::save_to_file`) and the extract path (`archive::extract_archive`). Isolating it means `archive.rs` depends only on a neutral type, not on the download/write module.

*Alternative — keep `Destination` with `save_to_file` in `fs.rs`:* rejected. It would force `archive.rs` to import the writer module just to name the type, which reads backwards ("the extractor depends on the downloader").

### D2: Filesystem side-effects live in `fs.rs`
`save_to_file` moves to `src/fs.rs`. Its cohesion is "perform a filesystem side-effect on a resolved path," which is also the future home of `set_executable`. This keeps side-effecting operations together and out of the extractor.

*Alternative — one combined module for all three (`Destination` + `save_to_file` + future `set_executable`):* rejected in favor of the neutral-type isolation in D1.

*Note on naming:* `crate::fs` visually echoes `std::fs` (which it uses internally). Accepted as the most honest umbrella for write + permission operations; `use std::fs` inside the module disambiguates.

### D3: `archive.rs` becomes extraction-only
`archive.rs` retains `is_extractable`, `extract_archive`, `unpack_tar_gz`, `extract_archive_entry`, `normalize_entry_path`, and adds `use crate::destination::Destination`. Everything not about tar.gz leaves.

### D4: Tests move with their code
Each relocated item's unit tests move into the module that now owns it (`destination.rs` tests for `Destination::resolve`; `fs.rs` tests for `save_to_file`; extraction tests stay in `archive.rs`). Tests are relocated verbatim, not rewritten, so `just test` continues to prove behavior is unchanged.

### Resulting dependency graph
Arrows point from a module to what it depends on (imports).
```
                    main.rs
            ┌──────────┼───────────┐
            ▼          ▼           ▼
          fs.rs    archive.rs   github.rs
            └────┐      │    ┌──────┘
                 ▼      ▼    │
              destination.rs │
                    │        │
                    ▼        ▼
                     error.rs
```
Both `fs.rs` and `archive.rs` depend on `destination::Destination`; everything bottoms out at `error.rs`.

## Risks / Trade-offs

- **A move can accidentally change behavior (visibility, imports)** → Keep items `pub` exactly as before; rely on the existing test suite (relocated unchanged) plus `just test` / `just lint` / `just format` to confirm parity.
- **`crate::fs` vs `std::fs` confusion** → Accepted; documented in D2, disambiguated by `use std::fs` at call sites.
