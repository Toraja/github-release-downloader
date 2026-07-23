# github-latest-release-downloader

A CLI tool that downloads assets from the latest release of any GitHub repository.
Given a repository URL and a regex pattern, it fetches the release metadata, matches assets by name, and either saves the file to disk or streams and extracts a `.tar.gz` archive in-place.

## Features

- Downloads the latest release asset matching a regex pattern
- Streams and extracts `.tar.gz`/`.tgz` archives without writing the archive to disk
- Supports authenticated requests via `GITHUB_TOKEN` to avoid rate limiting
- Flexible output path control with `--dir` and `--output` flags

## Installation

```sh
cargo install --path .
```

## Usage

```
github-latest-release-downloader <URL> <PATTERN> [OPTIONS]
```

See `github-latest-release-downloader --help` for more details.

## Releasing

Releases are automated via [cargo-dist](https://axodotdev.github.io/cargo-dist). Pushing a version tag triggers the GitHub Actions workflow, which builds binaries for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`, then publishes them as a GitHub Release.

```sh
# Ensure the version in Cargo.toml is updated first, then:
git tag github-latest-release-downloader-v<VERSION>
git push origin github-latest-release-downloader-v<VERSION>
```

The release appears at https://github.com/Toraja/cde/releases with the following artifacts per target:

- `github-latest-release-downloader-<TARGET>.tar.xz` — binary archive
- `github-latest-release-downloader-<TARGET>.tar.xz.sha256` — checksum
