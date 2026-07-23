## 1. Project Setup

- [x] 1.1 Add `clap` dependency to `Cargo.toml` with derive feature

## 2. CLI Argument Parsing

- [x] 2.1 Define CLI struct in `src/main.rs` using `clap` derive macros with a single positional `url` argument

## 3. URL Conversion Logic

- [x] 3.1 Implement a function `to_api_url(url: &str) -> Result<String, String>` that parses and converts the GitHub repo URL
- [x] 3.2 Handle trailing slashes by trimming before parsing
- [x] 3.3 Validate that the host is `github.com`
- [x] 3.4 Validate that the path contains both owner and repo segments (at least two non-empty path segments)
- [x] 3.5 Return the constructed `https://api.github.com/repos/<owner>/<repo>/releases/latest` string on success

## 4. Main Entry Point

- [x] 4.1 Wire CLI parsing to `to_api_url` in `main`, print result to stdout on success
- [x] 4.2 On error, print error message to stderr and exit with non-zero code

## 5. Tests

- [x] 5.1 Add unit tests for `to_api_url` covering: standard URL, trailing slash URL, non-GitHub domain, missing repo segment

## 6. Refactor: Use `url::Url` for CLI argument

- [x] 6.1 Add `url` crate dependency to `Cargo.toml`
- [x] 6.2 Change `Cli.url` field type from `String` to `url::Url`
- [x] 6.3 Update `to_api_url` signature to accept `&url::Url` instead of `&str` and simplify validation logic to use parsed URL fields
- [x] 6.4 Update call site in `main` to pass `&cli.url` directly
- [x] 6.5 Update unit tests to construct `url::Url` values via `Url::parse` instead of raw strings
