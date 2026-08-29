# add-vsix-support

## Why

Installing a VSCode extension (`.vsix`) manually currently requires a multi-step shell sequence: download the vsix asset, unzip it, and move the extracted `extension/` directory into the right place. A thin, opinionated subcommand collapses that into one `ghrls vsix` invocation (GitHub issue #8).

## What Changes

- New `vsix <URL> <PATTERN> [-D/--dir <DIR>]` subcommand. It fetches the matched asset, verifies its name ends with `.vsix`, and extracts the *contents of* the archive entry `extension/` directly to the destination — the `extension/` wrapper itself is stripped. It's a thin wrapper over `download --extract --archive-entry extension <URL> <PATTERN>` with a hard-coded entry name plus vsix-specific validation.
- Destination control on `vsix` supports only `-D/--dir`. The `--dir` flag denotes the destination directory itself (no `extension/` subfolder is created); omitting it means the current working directory. `--output` is rejected because renaming-vs-dir semantics here are uniform: the destination is always treated as the exact directory into which contents land.
- The archive module gains zip-extraction support (zip requires `Read + Seek`, so the full archive is buffered in memory — `Vec<u8>` → `Cursor` — and never written to disk). This is the only genuinely new extraction machinery needed; vsix happens to be zip, so it uses a zip-capable code path.
- `--output` never applies to vsix: the result is a directory (`extension/`) landing directly in `--dir`.

## Capabilities

### New Capabilities

- `vsix-install`: Specifies the `vsix` subcommand: matching a release asset by pattern, validating the `.vsix` extension, hard-coding extraction of the archive entry `extension/` (vsix-specific metadata files such as `extension.vsixmanifest` are discarded deliberately), and destination handling restricted to `--dir`.

### Modified Capabilities

- `asset-extraction`: The extraction capability gains archive-format routing and in-memory buffered zip extraction alongside the existing streaming tar.gz/tgz path. (The capability's `--extract` eligibility gate extends to zips.)

## Impact

- **Dependencies**: adds the `zip` crate (with `deflate` feature) to `Cargo.toml`.
- **Code**: `src/archive.rs` (zip extraction path, format routing), `src/main.rs` (new `Vsix` subcommand args, validation, run arm), `src/error.rs` (zip-specific and `.vsix` validation error variants).
- **Docs**: `README.md` gains a `vsix` section; `CHANGELOG.md` records the addition.
- **Tests**: new spec/test coverage for the `vsix` subcommand and the zip extraction path.