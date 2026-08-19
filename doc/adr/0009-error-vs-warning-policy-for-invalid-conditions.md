---
number: 9
title: Error vs warning policy for invalid conditions
status: accepted
date: 2026-08-19
---

# Error vs warning policy for invalid conditions

## Context and Problem Statement

As the CLI grows, it encounters two distinct kinds of "something is not right" conditions and needs a consistent rule for how each is surfaced:

1. User misuse — the invocation itself is wrong or contradictory (e.g. `--dir` together with `--output`, `--output` with whole-archive extraction, an `--archive-entry` value that matches no entry, an `--archive-entry` that resolves to a symlink, an unsupported archive format, or requesting `--executable` on a directory entry).
2. Archive-internal conditions the user cannot control — properties of the fetched archive's contents that are neither requested nor preventable by the caller (e.g. a child symlink encountered while extracting a directory entry).

Without an explicit policy, each new flag or edge case invites an ad-hoc choice between failing and continuing, producing an inconsistent CLI where equivalent situations sometimes abort and sometimes silently proceed.

## Decision Drivers

* Exit-code semantics must be predictable so scripts can distinguish genuine failures from benign noise.
* Contributors need a single, teachable rule when adding new flags and edge cases, rather than an ad-hoc choice each time.
* The policy should match behaviour that already exists in the codebase, not contradict it.
* Unavoidable properties of a fetched archive should not break otherwise-successful automation.

## Considered Options

* Classify by cause: user misuse → error, archive-internal noise → warn + continue
* Treat every invalid condition as a hard error
* Treat every invalid condition as a warning and always continue

## Decision Outcome

Chosen option: "Classify by cause", because it keeps exit codes meaningful (failures the caller can fix vs. quirks they cannot) and codifies the behaviour the CLI already exhibits.

Classify every invalid condition into one of two categories and handle each consistently:

- User misuse → hard error: print a descriptive message to stderr and exit non-zero. Where the mistake is detectable before any network request (argument conflicts), fail before making the request. Where it can only be detected after work has begun (e.g. `--executable` targeting a directory, only known after extraction), complete the work that did succeed, print its success line, then emit the error and exit non-zero (partial success is acceptable and must be communicated).
- Archive-internal noise the user cannot control → warn and continue: print a warning to stderr for the affected entry and proceed, exiting 0 on overall success.

The distinguishing test: could the user have avoided this by invoking the command differently? If yes, it is misuse and must error. If the condition is an intrinsic property of the fetched archive that the user neither asked for nor could prevent, warn and continue.

### Consequences

- Good, because it gives contributors a single rule to apply when adding flags or edge cases, keeping exit-code semantics predictable across the CLI.
- Good, because scripts can rely on non-zero exit codes signalling genuine user mistakes, while unavoidable archive quirks (like skipped symlinks) do not break automation.
- Good, because it matches the existing behaviour: misuse cases (`--dir`+`--output`, `--output`+whole-archive, entry-not-found, symlink-as-entry, unsupported-format) already error, while child-symlink skips during directory extraction already warn and continue.
- Bad, because the boundary can be debatable for future conditions; some cases may require a judgement call about whether the user "could have avoided it".
- Neutral, because post-work misuse errors (e.g. `--executable` on a directory) leave successfully-written files on disk alongside a non-zero exit; this partial-success outcome is intentional and must be spelled out in the error message so users do not re-run unnecessarily.

### Confirmation

Confirmed via code review that each invalid condition is handled per its category, and via tests: misuse cases assert a non-zero exit / error variant (argument conflicts, entry-not-found, symlink-as-entry, unsupported-format), while archive-internal cases assert a warning plus continued extraction and overall success (child-symlink skip during directory extraction).

## Pros and Cons of the Options

### Treat every invalid condition as a hard error

* Good, because it is maximally strict and unambiguous.
* Bad, because unavoidable archive quirks (e.g. a stray child symlink) would abort otherwise-successful extractions, breaking automation the user cannot fix.

### Treat every invalid condition as a warning and always continue

* Good, because the command rarely aborts.
* Bad, because genuine misuse would exit 0, hiding mistakes from scripts and users until they surface later, far from the cause.

## More Information

Applies to, and is exercised by, the `asset-extraction`, `extract-entry`, and `download-location` capabilities. Related: ADR 0008 (Error type representation) provides the typed `AppError` enum used to signal the error category.
