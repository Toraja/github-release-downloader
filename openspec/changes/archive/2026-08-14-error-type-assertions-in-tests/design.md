## Context

See proposal.md — Why. The two affected test modules are `src/github.rs` and `src/main.rs`. The project's error type is a single `AppError` enum in `src/error.rs` (using `thiserror`). The project targets stable Rust (currently 1.97); `std::assert_matches!` stabilised in Rust 1.82 and is available without any additional dependency.

## Goals / Non-Goals

**Goals:**
- All `is_err()` assertions replaced with checks that verify the concrete error type/variant
- Clap tests confirm `ErrorKind::ArgumentConflict` specifically
- `AppError` tests match on the exact variant and, where meaningful, the inner value
- Remove the redundant `test_invalid_regex` test

**Non-Goals:**
- Rewriting error message string-match assertions (`unwrap_err().to_string().contains(...)`) — those are already in place and acceptable
- Changing any production code
- Adding new test cases beyond what is needed to replace the weakened assertions

## Decisions

### Use `assert_matches!` for `AppError` variants

`assert_matches!(expr, pattern)` (imported via `use std::assert_matches;`) is the idiomatic way to match an enum variant with a guard in a test assertion. It produces a clear panic message on failure (shows the actual value) and avoids the verbose `match`/`unwrap_err` chain.

Alternatives considered:
- `matches!(…)` inside `assert!(…)` — works but gives a less informative panic message ("assertion failed" with no actual value shown)
- `unwrap_err()` + `if let` — verbose and does not short-circuit cleanly in tests

### Check `ErrorKind` for clap tests

`clap::error::Error::kind()` returns the `ErrorKind` enum. `ErrorKind::ArgumentConflict` is the appropriate variant for `conflicts_with` violations. This confirms not just that clap rejected the input but *why* it did.

Alternatives considered:
- Checking `.to_string()` for a known substring — fragile against clap version changes
- Keeping `is_err()` — insufficient; does not verify the error source

### Match inner values in `AppError` assertions

For `AppError::InvalidHost` and `AppError::InvalidPath` the inner string is a meaningful part of the contract (it carries the offending value). The guard clause in `assert_matches!` (`if inner == "..."`) verifies this without additional assertions.

## Risks / Trade-offs

- `std::assert_matches` is a stable feature since 1.82, so no risk on current toolchain. If the project ever needed to support an older MSRV, this would need a dev-dependency on the `assert_matches` crate instead — unlikely given edition 2024 is already in use.
- Matching on inner string values of `AppError` variants makes tests slightly more coupled to the exact string stored in the error. This is acceptable: the inner value is the host/path that triggered the error and is part of the error's contract.
- `#[derive(Debug)]` was added to `Args` to enable `.unwrap_err()` in the clap tests (which requires `T: Debug`). This is a minor production code change that was not anticipated in the proposal; it is benign and follows standard practice for clap structs.
