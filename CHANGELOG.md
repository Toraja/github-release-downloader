# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-08-22

### Added

- Unix-only `--executable` flag to set the executable bit on a downloaded file or a single extracted file.
- Shell completion generation via `ghrls completion <SHELL>` for bash, zsh, fish, powershell, and elvish.
- Error out early when the `--dir` path points to an existing file.

### Changed

- **Breaking:** Rename the binary from `github-release-downloader` to `ghrls`.
- **Breaking:** Require the `download` subcommand for downloads: `ghrls download <URL> <PATTERN>`.
- Change the default install path of the install script to `~/.local/bin`.

### Fixed

- Correct the unsupported-archive error message to reference `--extract`.

## [0.2.0] - 2026-08-18

### Changed

- **Breaking:** Replace `--extract-entry <ENTRY>` with `--extract --archive-entry <ENTRY>` and unify archive extraction paths.
- Allow `--output` together with `--extract` when `--archive-entry` is present, to rename the extracted entry

## [0.1.0] - 2026-08-17

### Added

- Download the latest release asset matching a regex pattern from any GitHub repository.
- Stream and extract `.tar.gz`/`.tgz` archives without writing the archive to disk (`--extract`).
- Narrow extraction to a single file or directory entry within the archive (`--extract-entry`).
- Rename extracted entries at the destination with `--output`.
- Authenticated requests via `GITHUB_TOKEN` to avoid rate limiting.
- Output path control with `--dir` and `--output` flags.

---

[Unreleased]: https://github.com/Toraja/github-release-downloader/compare/0.3.0...HEAD
[0.3.0]: https://github.com/Toraja/github-release-downloader/compare/v0.2.0...0.3.0
[0.2.0]: https://github.com/Toraja/github-release-downloader/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Toraja/github-release-downloader/releases/tag/v0.1.0
