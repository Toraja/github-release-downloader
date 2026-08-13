---
number: 5
title: tar.gz extraction method
status: accepted
date: 2026-06-09
---

# tar.gz extraction method

## Context and Problem Statement

Many GitHub release assets are `.tar.gz` archives containing a single binary. Users routinely followed a download with a manual `tar xzf` invocation to get the binary. Adding an `--extract` flag requires choosing how to perform tar.gz extraction in Rust.

## Decision Drivers

* Extraction must be portable across platforms including Windows.
* Error handling should be idiomatic Rust without process-spawn complexity.
* Prefer well-established ecosystem crates over custom or system-level solutions.

## Considered Options

* `flate2` + `tar` crates
* Shell out to the system `tar` binary

## Decision Outcome

Chosen option: "`flate2` + `tar` crates", because both are the de facto standard for tar.gz handling in the Rust ecosystem, they are portable across platforms including Windows, and they allow idiomatic error handling without process-spawn complexity.

### Confirmation

Confirmed via code review that `flate2::read::GzDecoder` and `tar::Archive` are used for extraction and no `std::process::Command` invocation of `tar` exists.

## Pros and Cons of the Options

### `flate2` + `tar` crates

* Good, because de facto standard crates; portable across platforms including Windows.
* Good, because idiomatic Rust error handling without process-spawn complexity.
* Good, because `tar::unpack()` mitigates path traversal attacks by skipping absolute paths and `..` components.
* Bad, because adds two dependencies, though both are widely used and stable.

### Shell out to the system `tar` binary

* Good, because no additional Rust dependencies.
* Bad, because not portable — `tar` is not guaranteed to be available on Windows.
* Bad, because adds process-spawn complexity and makes error handling harder.
