## 1. Update Function Signature and Body

- [x] 1.1 Change `to_api_url` return type from `Result<String, String>` to `Result<url::Url, String>`
- [x] 1.2 Replace the `Ok(format!(...))` return with `Url::parse(&format!(...)).map_err(|e| e.to_string())`

## 2. Update Call Sites

- [x] 2.1 Update `main()` to call `.to_string()` on the returned `url::Url` before passing to `println!`

## 3. Update Tests

- [x] 3.1 Update `test_standard_url` assertion to compare against `Ok(Url::parse("https://api.github.com/repos/owner/repo/releases/latest").unwrap())`
- [x] 3.2 Update `test_trailing_slash_url` assertion similarly
- [x] 3.3 Verify `test_non_github_domain` and `test_missing_repo_segment` still pass unchanged (they only check `is_err()`)

## 4. Verify

- [x] 4.1 Run `cargo build` with no errors
- [x] 4.2 Run `cargo test` with all tests passing
