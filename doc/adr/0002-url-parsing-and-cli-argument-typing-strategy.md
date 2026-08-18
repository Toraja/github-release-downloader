---
number: 2
title: URL parsing and CLI argument typing strategy
status: accepted
date: 2026-05-11
---

# URL parsing and CLI argument typing strategy

## Context and Problem Statement

The CLI accepts a GitHub repository URL as an argument. The URL must be validated and its components (host, path segments) extracted to construct the API URL. A decision is needed on whether to parse URLs manually or use a dedicated crate, and whether to type the CLI argument as `String` or `url::Url`.

## Decision Drivers

* URL validation must happen before any application logic runs.
* URL components (host, path segments) must be reliably extractable.
* Correct handling of scheme, percent-encoding, and trailing slashes is required.
* Prefer using proven libraries over reimplementing URL parsing logic.

## Considered Options

* `url` crate with `url::Url` as the CLI argument type
* Manual parsing with `split('/')`

## Decision Outcome

Chosen option: "`url` crate with `url::Url` as the CLI argument type", because typing the argument as `url::Url` lets `clap` validate URL format automatically before any application logic runs, and the `to_api_url` function can work directly with the parsed struct via `.host_str()` and `.path_segments()`.

### Confirmation

Confirmed via code review that `Args.url` is typed as `url::Url` and that `to_api_url` uses the parsed struct methods rather than string splitting.

<!-- This is an optional element. Feel free to remove. -->
## Pros and Cons of the Options

### `url` crate with `url::Url` as the CLI argument type

* Good, because invalid URLs are rejected by `clap` before any application logic runs — no manual pre-validation needed.
* Good, because `to_api_url` works directly with the parsed struct (`.host_str()`, `.path_segments()`), avoiding fragile string splitting.
* Good, because percent-encoding, scheme handling, and trailing slash normalisation are handled correctly by the library.
* Bad, because adds the `url` crate as a dependency, though it is widely used and stable.

### Manual parsing with `split('/')`

* Good, because no additional dependencies.
* Bad, because requires reimplementing scheme handling, percent-encoding, and trailing slash trimming that the `url` crate already provides correctly.
* Bad, because URL validation must be done manually and is error-prone.
