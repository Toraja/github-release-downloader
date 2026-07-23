## 1. Dependencies

- [x] 1.1 Add `ureq` with `json` feature to `Cargo.toml`
- [x] 1.2 Add `serde` with `derive` feature and `serde_json` to `Cargo.toml`
- [x] 1.3 Add `regex` to `Cargo.toml`

## 2. CLI Arguments

- [x] 2.1 Add a second positional argument `pattern` (regex string) to the `Cli` struct
- [x] 2.2 Add help text mentioning `GITHUB_TOKEN` env var for authenticated requests

## 3. Regex Validation

- [x] 3.1 Compile the user-provided pattern with `regex::Regex::new` before any HTTP request
- [x] 3.2 Exit with a non-zero code and clear stderr message if the regex is invalid

## 4. GitHub API Fetch

- [x] 4.1 Define `Asset` struct with `name` and `browser_download_url` fields, derived `Deserialize`
- [x] 4.2 Define `Release` struct with an `assets: Vec<Asset>` field, derived `Deserialize`
- [x] 4.3 Build the API request using `ureq`; include `Authorization: Bearer <token>` header if `GITHUB_TOKEN` is set
- [x] 4.4 Handle HTTP/network errors and exit with a non-zero code and stderr message

## 5. Asset Matching

- [x] 5.1 Filter `release.assets` to those whose `name` matches the compiled regex
- [x] 5.2 If zero matches: exit non-zero, print error to stderr listing all available asset names
- [x] 5.3 If more than one match: exit non-zero, print error to stderr listing all matching asset names
- [x] 5.4 If exactly one match: proceed with download

## 6. Asset Download

- [x] 6.1 Make an HTTP GET request to `browser_download_url`
- [x] 6.2 Stream the response body into a file in the current working directory named after the asset
- [x] 6.3 Handle download errors and exit with a non-zero code and stderr message

## 7. Remove Old Stdout Output

- [x] 7.1 Remove the `println!` that previously printed the API URL to stdout

## 8. Tests

- [x] 8.1 Add unit test: invalid regex exits before HTTP request
- [x] 8.2 Add unit test: zero matching assets returns correct error
- [x] 8.3 Add unit test: multiple matching assets returns correct error
- [x] 8.4 Add unit test: exactly one matching asset is selected correctly
