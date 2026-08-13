---
number: 7
title: Archive format detection strategy
status: accepted
date: 2026-06-09
---

# Archive format detection strategy

## Context and Problem Statement

When `--extract` is specified, the tool must verify that the matched asset is an extractable archive format. A decision is needed on when and how to detect the format: by inspecting the asset filename extension before downloading, or by reading magic bytes after downloading.

## Decision Drivers

* Invalid format should be rejected as early as possible to avoid wasting bandwidth.
* The asset filename is already available from the API response before any download begins.
* Detection logic should be simple and predictable.

## Considered Options

* Detect format by file extension before downloading
* Detect format by magic bytes after downloading

## Decision Outcome

Chosen option: "Detect format by file extension before downloading", because the asset filename is already available from the GitHub API response, allowing a fast fail with a clear error message before any download bandwidth is consumed.

### Confirmation

Confirmed via code review that format validation occurs against the asset filename before the HTTP download request is made.

## Pros and Cons of the Options

### Detect format by file extension before downloading

* Good, because fails fast with a clear error message before downloading — no wasted bandwidth.
* Good, because simple and predictable: the filename is already available from the API response.
* Bad, because relies on the asset filename being correctly named; a misnamed file would pass or fail incorrectly.

### Detect format by magic bytes after downloading

* Good, because accurate regardless of how the file is named.
* Bad, because requires downloading the asset before the format can be validated — wastes bandwidth on unsupported formats.
* Bad, because more complex: requires buffering the start of the response to read magic bytes.
