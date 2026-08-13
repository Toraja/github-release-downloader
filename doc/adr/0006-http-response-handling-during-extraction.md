---
number: 6
title: HTTP response handling during extraction
status: accepted
date: 2026-06-09
---

# HTTP response handling during extraction

## Context and Problem Statement

When extracting a `.tar.gz` asset, the HTTP response body must be fed into the decompressor and archive reader. A decision is needed on whether to buffer the response to a temporary file first or pipe it directly into the extractor.

## Decision Drivers

* Large binary assets must not require unnecessary disk I/O.
* The implementation should be as simple as possible.
* The HTTP response body already implements `Read`, making streaming natural.

## Considered Options

* Stream HTTP response directly through `GzDecoder` into `tar::Archive`
* Download to a temporary file first, then extract from the file

## Decision Outcome

Chosen option: "Stream HTTP response directly through `GzDecoder` into `tar::Archive`", because the HTTP response body implements `Read` and can be fed directly into `flate2::read::GzDecoder` and then `tar::Archive::unpack()`, avoiding unnecessary disk I/O and temporary file management complexity.

### Confirmation

Confirmed via code review that the extraction path pipes the `ureq` response body directly into `GzDecoder` without writing a temporary file.

## Pros and Cons of the Options

### Stream HTTP response directly through `GzDecoder` into `tar::Archive`

* Good, because avoids unnecessary disk I/O — no temp file written and deleted.
* Good, because simpler code with no temp file lifecycle to manage.
* Bad, because on extraction failure, there is no local copy of the archive to retry from.

### Download to a temporary file first, then extract from the file

* Good, because allows retrying extraction without re-downloading.
* Bad, because requires unnecessary disk I/O for the temp file write and subsequent delete.
* Bad, because adds temp file lifecycle management complexity.
