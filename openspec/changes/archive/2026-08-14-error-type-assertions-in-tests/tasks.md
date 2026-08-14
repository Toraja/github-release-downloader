## 1. src/github.rs — Test cleanup

- [x] 1.1 Remove the `test_invalid_regex` test function (including its `#[test]` and `#[allow(clippy::invalid_regex)]` attributes)
- [x] 1.2 Add `use std::assert_matches;` to the test module imports
- [x] 1.3 Add `use crate::error::AppError;` to the test module imports
- [x] 1.4 Replace `assert!(to_api_url(&url).is_err())` in `test_non_github_domain` with `assert_matches!(to_api_url(&url), Err(AppError::InvalidHost(h)) if h == "gitlab.com")`
- [x] 1.5 Replace `assert!(to_api_url(&url).is_err())` in `test_missing_repo_segment` with `assert_matches!(to_api_url(&url), Err(AppError::InvalidPath(p)) if p == "/owner")`

## 2. src/main.rs — Clap error kind assertions

- [x] 2.1 Add `use clap::error::ErrorKind;` to the test module imports
- [x] 2.2 Replace `assert!(result.is_err())` in `test_dir_and_output_mutually_exclusive` with `assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict)`
- [x] 2.3 Replace `assert!(result.is_err())` in `test_extract_and_output_mutually_exclusive` with `assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict)`
- [x] 2.4 Replace `assert!(result.is_err())` in `test_extract_entry_and_extract_mutually_exclusive` with `assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict)`

## 3. Verification

- [x] 3.1 Run `just test` and confirm all tests pass
- [x] 3.2 Run `just lint` and confirm no warnings
- [x] 3.3 Run `just format` and confirm no formatting changes
