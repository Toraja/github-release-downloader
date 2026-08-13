---
number: 1
title: CLI argument parsing library
status: accepted
date: 2026-05-11
---

# CLI argument parsing library

## Context and Problem Statement

This is a new Rust CLI tool (`github-release-downloader`). A library for parsing command-line arguments must be chosen. The tool is expected to grow with additional flags over time.

## Decision Drivers

* The tool must provide user-friendly help text and error messages out of the box.
* The tool is expected to grow with additional flags and subcommands over time, requiring minimal boilerplate for each addition.
* Type-safe argument handling is preferred to reduce runtime errors.
* Prefer ecosystem-standard libraries to benefit from community support and maintenance.

## Considered Options

* clap
* std::env::args()

## Decision Outcome

Chosen option: "clap", because it is the de facto standard for Rust CLI tools and provides help text, error messages, and shell completion for free via `#[derive(Parser)]`.

### Confirmation

Confirmed via code review that `clap` is used for argument parsing (e.g., `#[derive(Parser)]` on the args struct).

<!-- This is an optional element. Feel free to remove. -->
## Pros and Cons of the Options

### clap

* Good, because self-documenting CLI with help text and type-safe argument handling at zero extra code cost.
* Good, because adding new flags and subcommands in future requires minimal boilerplate.
* Bad, because adds a dependency, though `clap` is the ecosystem standard and widely used.

### std::env::args()

* Good, because no additional dependencies.
* Bad, because too manual for a CLI expected to grow with additional flags and subcommands.
* Bad, because no built-in help text, error messages, or shell completion.
