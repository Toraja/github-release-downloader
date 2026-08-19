## Why

Shell completion improves usability by letting users tab-complete flags and arguments without consulting the help text. Clap's `clap_complete` crate can generate completion scripts for common shells at compile time or on demand, so the cost to add this is low.

## What Changes

- Add `clap_complete` as a build dependency.
- Add a `--generate-completion <SHELL>` flag (or a `completion` subcommand) to the CLI that prints a shell completion script to stdout and exits.
- Supported shells: Bash, Zsh, Fish, PowerShell, Elvish (whatever `clap_complete::Shell` provides).

## Capabilities

### New Capabilities

- `shell-completion`: CLI accepts a request to generate and print a shell completion script for a given shell, then exits without performing any download.

### Modified Capabilities

<!-- No existing spec-level behavior changes. -->

## Impact

- `Cargo.toml`: add `clap_complete` dependency.
- `src/main.rs`: extend `Args` (or add a subcommand) to handle the completion generation request; call `clap_complete::generate()` and exit before normal argument validation.
- No changes to download, extraction, or GitHub API logic.
