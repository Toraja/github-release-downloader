## Context

The tool is a Rust CLI that accepts a GitHub repository URL and currently converts it to the GitHub API latest release URL, printing it to stdout. The URL conversion logic (`to_api_url`) already exists and is correct. The next step is to use that URL to fetch release metadata via the GitHub API, apply regex-based filtering to select exactly one asset, and download it.

The project is a single-file Rust binary (`src/main.rs`) using `clap` for argument parsing and `url` for URL handling.

## Goals / Non-Goals

**Goals:**
- Accept a second CLI argument: a regex pattern to match against asset names.
- Fetch the GitHub API latest release JSON using a synchronous HTTP client.
- Match asset names against the regex; error if zero or more than one asset matches.
- Download the single matched asset binary to the current working directory.
- Optionally authenticate via a `GITHUB_TOKEN` environment variable to raise API rate limits.

**Non-Goals:**
- Downloading to a custom output directory (future work).
- Downloading multiple assets in one invocation.
- Handling release types other than "latest" (e.g., pre-releases, specific tags).
- Resumable downloads or progress bars.

## Decisions

### HTTP client: `ureq`
Since only one asset is downloaded per invocation and all steps are sequential, a synchronous HTTP client is sufficient and simpler. `ureq` is lightweight, has no async runtime dependency, and supports streaming response bodies directly to a writer — avoiding loading large binaries into memory. `reqwest` (async) was considered but rejected: async adds complexity with no benefit when there is nothing to do concurrently.

### JSON deserialization: `serde` + `serde_json`
The GitHub API returns JSON. `serde` with `#[derive(Deserialize)]` gives typed, zero-boilerplate deserialization. Only the fields we need (`assets[].name`, `assets[].browser_download_url`) are extracted, ignoring the rest.

### Regex crate: `regex`
The standard `regex` crate is used for asset name matching. The user-supplied pattern is compiled once and matched against each asset name. This handles version strings naturally (e.g., `gh_.*_linux_amd64` matches `gh_2.40.1_linux_amd64.tar.gz`).

### Exactly-one-match enforcement
After filtering, the matched asset count is checked:
- 0 matches → error with list of available asset names.
- 2+ matches → error with list of all matching asset names, so the user can refine the pattern.
- 1 match → proceed to download.

### Authentication: `GITHUB_TOKEN` env var
If `GITHUB_TOKEN` is set, it is passed as an `Authorization: Bearer <token>` header. This is the standard approach used by `gh` CLI and GitHub Actions. No CLI flag is added to avoid leaking tokens via shell history.

### Output filename
The asset is saved using its original name from the API response, in the current working directory.

## Risks / Trade-offs

- **Large asset downloads with no progress feedback** → Acceptable for now; a future enhancement can add a progress bar via `indicatif`.
- **Regex compilation failure on bad user input** → Mitigation: compile the regex before making any HTTP request and exit early with a clear error message.
- **GitHub API rate limiting for unauthenticated users (60 req/hr)** → Mitigation: document `GITHUB_TOKEN` usage in CLI help text.
