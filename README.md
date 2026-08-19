# github-release-downloader

A CLI tool that downloads assets from the latest release of any GitHub repository.
Given a repository URL and a regex pattern, it fetches the release metadata, matches assets by name, and either saves the file to disk or streams and extracts a `.tar.gz` archive in-place.

## Features

- Downloads the latest release asset matching a regex pattern
- Streams and extracts `.tar.gz`/`.tgz` archives without writing the archive to disk (`--extract`)
- Optionally narrows extraction to a single file or directory entry within the archive (`--archive-entry`)
- Extracted entries can be renamed at the destination with `--output`
- Supports authenticated requests via `GITHUB_TOKEN` to avoid rate limiting
- Flexible output path control with `--dir` and `--output` flags

## Installation

### Use install script

This will install the latest release binary to `$HOME/.local/bin`.

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Toraja/github-release-downloader/releases/latest/download/github-release-downloader-installer.sh | sh
```

### Build from source

```sh
cargo install --path .
```

## Usage

```
ghrls download <URL> <PATTERN> [OPTIONS]
```

See `ghrls download --help` for all options.

To generate a shell completion script:

```
ghrls completion <SHELL>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

Example — add bash completions to your current session:

```sh
source <(ghrls completion bash)
```

See `ghrls --help` for more details.

## Releasing

Releases are automated via [cargo-dist](https://axodotdev.github.io/cargo-dist).
Pushing a version tag triggers the GitHub Actions workflow, which builds binaries, then publishes them as a GitHub Release.

To tag a new release, run:
```sh
just release-execute <level|version>
```
