# github-release-downloader (ghrls)

## Features

- Downloads the latest release asset matching a regex pattern
- Prints the latest release tag, with optional literal prefix removal
- Streams and extracts `.tar.gz`/`.tgz` archives without writing the archive to disk (`--extract`)
- Optionally narrows extraction to a single file or directory entry within the archive (`--archive-entry`)
- Extracted entries can be renamed at the destination with `--output`
- Supports authenticated requests via `GITHUB_TOKEN` to avoid rate limiting
- Flexible output path control with `--dir` and `--output` flags

## Motivation

`ghrls` aims to simplify the process of downloading release assets from GitHub, especially in automated environments like Docker builds.

Whenever you want to download a release asset from GitHub, you need to:
1. Get the URL of the asset you want to download from the latest release info
1. Download the asset with `curl` or `wget`
1. (Optionally) Extract the asset and rename the extracted file
1. (Optionally) Set the executable bit on the downloaded file
1. (Optionally) Delete the downloaded asset

The command usually looks like:
```sh
curl https://api.github.com/repos/owner/repo/releases/latest | jq -r '.assets[] | select( .name | match ("^my-tool-x86_64-unknown-linux-gnu.tar.gz$") ) | .browser_download_url' | tar -xz --strip-components=1 -C /usr/local/bin`
```

This makes Dockerfile quite messy and hard to maintain.  
(I know downloading latest makes the build unreproducible, but I have usecases.)

`gh release download` simplifies it, but still requires extra steps for archive extraction and authentication is a must.
```sh
gh release download -R owner/repo -p my-tool-x86_64-unknown-linux-gnu.tar.gz -D /usr/local/bin | tar -xz --strip-components=1 -C /usr/local/bin

# For Dockerfile
RUN --mount=type=secret,id=GH_TOKEN,env=GH_TOKEN \
    gh release download ...
```

With `ghrls`, this will become:
```sh
ghrls download -x -X dir/mytool -D /usr/local/bin https://github.com/owner/repo my-tool-x86_64-unknown-linux-gnu.tar.gz
```

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

To print the latest release tag:

```sh
ghrls tag https://github.com/owner/repo
```

Use `--strip-prefix` to remove one literal, case-sensitive prefix when present.
This is useful for consuming repositories that inconsistently include a `v` prefix:

```sh
version=$(ghrls tag https://github.com/owner/repo --strip-prefix v)
```

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
