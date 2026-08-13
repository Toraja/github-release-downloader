---
number: 3
title: HTTP client selection
status: accepted
date: 2026-05-12
---

# HTTP client selection

## Context and Problem Statement

The tool fetches release metadata and downloads assets from the GitHub API. Since only one asset is downloaded per invocation and all steps are sequential, an async runtime is unnecessary. An HTTP client must be chosen.

## Decision Drivers

* Only one asset is downloaded per invocation and all steps are sequential, so an async runtime is unnecessary.
* Large binary assets must not be loaded fully into memory during download.

## Considered Options

* `ureq` (sync)
* `reqwest` (async)

## Decision Outcome

Chosen option: `ureq` (sync), because a synchronous HTTP client is sufficient for sequential single-asset downloads and `ureq` supports streaming to avoid loading large binaries into memory.

### Confirmation

Confirmed via code review that `ureq` is used for HTTP with streaming.

## Pros and Cons of the Options

### `ureq` (sync)

* Good, because `ureq` is lightweight with no async runtime overhead.
* Good, because streaming support avoids loading large binaries into memory.

### `reqwest` (async)

* Good, because `reqwest` is the most widely used Rust HTTP client with extensive documentation.
* Bad, because async adds complexity (requires a runtime such as `tokio`) with no benefit when there is nothing to do concurrently.
