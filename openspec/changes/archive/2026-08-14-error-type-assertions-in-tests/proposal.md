## Why

Test assertions that only check `is_err()` do not verify which error variant was returned, allowing incorrect error types to pass silently. Strengthening these assertions to match on specific error variants makes tests more precise and catches regressions where the right operation fails for the wrong reason.

## What Changes

- Remove `test_invalid_regex` in `src/github.rs` — it tests `regex::Regex::new` itself, not application logic, and is unnecessary
- Replace `is_err()` assertions in `src/github.rs` (`test_non_github_domain`, `test_missing_repo_segment`) with `assert_matches!` against specific `AppError` variants and their inner values
- Replace `is_err()` assertions in `src/main.rs` (three CLI mutual-exclusion tests) with `unwrap_err().kind() == ErrorKind::ArgumentConflict` to confirm errors originate from clap's argument conflict detection
- Add `use std::assert_matches;` to the `github.rs` test module

## Capabilities

### New Capabilities
<!-- None — this is a pure test refactor with no spec-level behavior changes -->

### Modified Capabilities
<!-- None -->

## Impact

- `src/github.rs` test module: remove one test, update two assertions
- `src/main.rs` test module: update three assertions, add `use clap::error::ErrorKind`
- No production code changes; no API or behavior changes
