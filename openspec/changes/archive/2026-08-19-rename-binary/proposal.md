## Why

The binary is currently named `github-release-downloader`. A planned `download` subcommand (see the `shell-completion` change) would read redundantly as `github-release-downloader download`. Renaming the binary to `ghrls` gives a short, memorable invocation (`ghrls download <URL> <PATTERN>`) ahead of the subcommand restructure.

## What Changes

- Rename the produced binary to `ghrls` via an explicit `[[bin]]` section in `Cargo.toml`.
- Keep the Cargo package name (`github-release-downloader`) and the GitHub repository unchanged, so the cargo-dist installer URL and existing install instructions keep working.
- Update the `User-Agent` header string and `README.md` usage examples to the new binary name.
- **BREAKING**: The installed executable is now `ghrls`; users who invoked `github-release-downloader` directly must use `ghrls`.

## Capabilities

This is a rename/packaging change with no spec-level behavior change (arguments, download, and extraction behavior are unchanged). No capabilities are added or modified; `skip_specs: true` is set in `.openspec.yaml`.

## Impact

- `Cargo.toml`: add `[[bin]]` with `name = "ghrls"`.
- `src/github.rs`: update `User-Agent` header strings (lines 48, 92).
- `README.md`: update usage examples and `--help` references.
- `Cargo.lock`: regenerates on next build.
- No change to package name, repository URL, cargo-dist config, or the installer script name.
