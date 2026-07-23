## Why

The tool currently converts a GitHub repository URL to an API URL and prints it, but stops short of doing anything useful. Users have to manually use the printed URL to fetch release metadata and download assets — the core purpose of a "latest release downloader" is not yet implemented.

## What Changes

- The CLI will accept an additional argument: a regex pattern to select which asset to download.
- It will fetch the GitHub API latest release endpoint (constructed from the input URL) using an HTTP client.
- It will parse the JSON response to extract the list of downloadable assets.
- It will match asset names against the regex and error if zero or more than one asset matches.
- It will download the single matching asset to the current working directory.
- The existing URL conversion logic is preserved and reused internally.

## Capabilities

### New Capabilities

- `asset-download`: Fetch the latest release metadata from the GitHub API, select exactly one asset whose name matches a user-provided regex pattern, and download it to disk. Zero matches or multiple matches are both errors.

### Modified Capabilities

- `url-conversion`: The URL conversion result is now used internally to make the HTTP request rather than being printed to stdout. The CLI's externally visible behavior changes — instead of printing the API URL, it downloads the matched asset.

## Impact

- `src/main.rs`: Major changes — add asset name regex filtering, HTTP client usage, JSON parsing, file I/O for asset download; remove stdout printing of the API URL.
- New dependencies: `reqwest` (async HTTP client with TLS), `tokio` (async runtime), `serde` / `serde_json` (JSON deserialization), `regex` (asset name pattern matching).
- GitHub API rate limits apply to unauthenticated requests (60/hour); authenticated requests via `GITHUB_TOKEN` env var should be supported optionally.
