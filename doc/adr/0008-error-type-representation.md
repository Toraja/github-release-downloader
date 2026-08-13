---
number: 8
title: Error type representation
status: accepted
date: 2026-08-12
---

# Error type representation

## Context and Problem Statement

Every fallible function in the codebase returned `Result<_, String>` with format strings scattered through `map_err` calls. This prevents pattern-matching on error kinds and requires every caller to write format strings manually.

## Decision Drivers

* Error kinds must be pattern-matchable for precise handling.
* Error formatting should not be scattered across every call site.
* The error type should be idiomatic Rust and easy to extend as the codebase grows.

## Considered Options

* Typed `AppError` enum via `thiserror`
* `String` errors with `map_err` format strings

## Decision Outcome

Chosen option: "Typed `AppError` enum via `thiserror`", because a single `AppError` enum in `src/error.rs` enables pattern-matching on error kinds and eliminates scattered `map_err` format strings across all call sites.

### Confirmation

Confirmed via code review that all fallible functions return `Result<_, AppError>` and no `Result<_, String>` remains in the codebase and tests that asserts error type matching.

## Pros and Cons of the Options

### Typed `AppError` enum via `thiserror`

* Good, because enables pattern-matching on error kinds.
* Good, because eliminates scattered `map_err` format strings — error messages are defined once on the enum variants.
* Bad, because single `AppError` enum covering all modules may need splitting if the codebase grows significantly.

### `String` errors with `map_err` format strings

* Good, because no additional dependencies.
* Bad, because prevents pattern-matching on error kinds.
* Bad, because requires every caller to write format strings, scattering error message logic across the codebase.
