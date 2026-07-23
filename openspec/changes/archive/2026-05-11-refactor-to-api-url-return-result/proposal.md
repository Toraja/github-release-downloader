## Why

`to_api_url` currently returns `Result<String, String>`, building the API URL as a plain string. Returning a typed `url::Url` makes the output self-validating, eliminates unnecessary string-to-URL re-parsing at call sites, and aligns with idiomatic Rust API design.

## What Changes

- **BREAKING**: `to_api_url` signature changes from `fn to_api_url(url: &Url) -> Result<String, String>` to `fn to_api_url(url: &Url) -> Result<url::Url, String>`
- The function body constructs and returns a `url::Url` instead of a `String`
- Call sites in `main` and tests are updated to work with the new return type (e.g., `.to_string()` for display, comparison against `Url` values in assertions)

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `url-conversion`: The internal return type of the conversion function changes; the observable behavior (URL format, error cases) remains the same, but the function now returns a typed `url::Url` value

## Impact

- `src/main.rs`: `to_api_url` function signature and body; call sites in `main()` and unit tests
- No new dependencies (the `url` crate is already used)
- No change to CLI behavior or output format
