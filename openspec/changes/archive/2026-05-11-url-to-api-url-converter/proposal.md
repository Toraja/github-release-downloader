## Why

The CLI needs a foundational step: converting a human-readable GitHub repository URL into the GitHub API latest release URL. Without this conversion, downstream asset downloading cannot begin. This iteration delivers that core URL transformation as a usable, verifiable output.

## What Changes

- Add CLI argument parsing to accept a GitHub repository URL
- Implement URL conversion logic from `https://github.com/<owner>/<repo>` to `https://api.github.com/repos/<owner>/<repo>/releases/latest`
- Output the converted API URL to stdout

## Capabilities

### New Capabilities
- `url-conversion`: Accepts a GitHub repo URL as input and outputs the corresponding GitHub API latest release URL

### Modified Capabilities

## Impact

- New CLI binary entry point in `src/main.rs`
- Introduces argument parsing (e.g., using `clap`)
- No external network calls in this iteration - pure URL transformation
