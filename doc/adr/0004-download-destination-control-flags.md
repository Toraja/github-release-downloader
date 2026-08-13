---
number: 4
title: Download destination control flags
status: accepted
date: 2026-05-14
---

# Download destination control flags

## Context and Problem Statement

The tool saved downloaded assets to the current working directory with no way to change the destination. Users scripting the tool or running it from CI environments needed a way to specify a destination without first changing directories. The change is confined to argument parsing and file-write layers.

## Decision Drivers

* The download destination must be controllable without changing the working directory.
* Semantics of destination flags must remain unambiguous and compose well with future features.

## Considered Options

* Two separate flags: `--dir` for destination directory and `--output` for exact file path
* Single `--output` flag with runtime dir-vs-file heuristic

## Decision Outcome

Chosen option: "Two separate flags: `--dir` for destination directory and `--output` for exact file path", because:
- Separate flags keep semantics unambiguous and interact correctly with the future tarball-extract feature, where the output is always a directory regardless of path shape
- Mutual exclusivity can be easily enforced by clap's `conflicts_with`.

### Confirmation

Confirmed via code review that `--dir` and `--output` are defined as separate flags with `conflicts_with` enforcing mutual exclusivity and tested that providing both flag results in an error.

<!-- This is an optional element. Feel free to remove. -->
## Pros and Cons of the Options

### Two separate flags: `--dir` for destination directory and `--output` for exact file path

* Good, because separate flags keep semantics unambiguous and compose naturally with future features via clap `conflicts_with`.
* Good, because default behaviour (current working directory) is unchanged; existing scripts are unaffected.
* Bad, because `--output` pointing to a directory produces a clap error; users expecting single-flag heuristics may be surprised.

### Single `--output` flag with runtime dir-vs-file heuristic

* Good, because simpler interface with a single flag for both use cases.
* Bad, because runtime heuristic (trailing slash / existing directory check) interacts poorly with the future tarball-extract feature, where the output is always a directory regardless of path shape.
* Bad, because ambiguous semantics make the behaviour harder to document and predict.
