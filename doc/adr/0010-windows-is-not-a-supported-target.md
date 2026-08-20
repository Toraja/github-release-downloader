---
number: 10
title: Windows is not a supported target
status: accepted
date: 2026-08-20
---

# Windows is not a supported target

## Context and Problem Statement

The CLI is a GitHub release-asset downloader. Its core behaviour — fetching a
release, matching an asset by regex, saving to disk, and extracting `.tar.gz`
archives — is already cross-platform. Some behaviour, however, is inherently
Unix-only: the forthcoming `--executable` flag sets the executable bit via
`std::os::unix::fs::PermissionsExt`, which has no meaning on Windows.

This forces a question the project has never stated explicitly: **is Windows a
supported target, and if not, what exactly does "not supported" mean?** The
phrase is ambiguous — it could mean the crate does not even compile on non-Unix
hosts, or it could mean the crate compiles and its cross-platform features work
while Unix-only behaviour is unavailable. Without an explicit stance, each
platform-conditional code path (`#[cfg]` guards) and each Unix-only feature
invites an ad-hoc choice, and the meaning of "not supported" drifts.

## Decision Drivers

* Contributors on a non-Unix host must be able to build the crate and run
  `just test` / `just lint` without the tree failing to compile.
* Only the executable bit is Unix-specific today; the rest of the tool is
  already portable, so a hard "does not compile" stance would be needlessly
  strict.
* There is no current demand for Windows users, and committing to Windows would
  add shipping, CI, and support-burden obligations we do not want now.
* Contributors need a single, teachable rule for what to do when a feature is
  Unix-only, rather than deciding case by case.

## Considered Options

* Not supported, meaning (ii): the crate compiles on non-Unix and cross-platform
  features work; Windows is not shipped or CI-tested; Unix-only behaviour
  degrades gracefully at runtime.
* Not supported, meaning (i): the crate does not compile on non-Unix hosts
  (e.g. `compile_error!` on any Unix-only path).
* Fully support Windows: ship and CI-test Windows builds and provide a
  Windows-appropriate alternative for every Unix-only behaviour.

## Decision Outcome

Chosen option: "Not supported, meaning (ii)", because it keeps the door open for
non-Unix contributors and preserves the tool's already-portable core without
taking on the burden of a committed Windows target.

Concretely, the project commitment is:

- Windows is **not shipped and not CI-tested**; the project makes **no
  correctness or support guarantees** on Windows.
- The crate **MUST still compile** on non-Unix hosts, and cross-platform
  features (fetch, match, save, `.tar.gz` extraction) are expected to work.
- Unix-only behaviour (today, the `--executable` bit) is confined behind
  `#[cfg(unix)]` and **degrades gracefully at runtime** on non-Unix rather than
  breaking the build. "Not supported" therefore means *unshipped and untested*,
  not *uncompilable*.

### Consequences

- Good, because contributors on non-Unix hosts can build, lint, and test the
  crate.
- Good, because it preserves the option to promote Windows to a best-effort
  target later without a rewrite, since the core is already portable.
- Good, because it gives a single rule for Unix-only features: guard with
  `#[cfg(unix)]` and provide a graceful non-Unix path.
- Bad, because "compiles and mostly works" can be mistaken for "supported"; the
  unshipped/untested status must be communicated so no support expectation forms.
- Neutral, because each Unix-only feature now carries a small design obligation
  to define its non-Unix degradation.
- Neutral, because there is no Windows/non-Unix CI, so a non-Unix build breakage
  could go unnoticed until a contributor hits it; adherence relies on review.

### Confirmation

The stance is upheld by confining Unix-only code behind `#[cfg(unix)]` guards
with a graceful non-Unix path, verified in code review. There is currently no
automated non-Unix build check, so compliance relies on reviewer judgement
rather than a fitness function.

## Pros and Cons of the Options

### Not supported, meaning (i): does not compile on non-Unix

* Good, because it is unambiguous — non-Unix is simply out of scope.
* Bad, because it blocks non-Unix contributors from building or testing the
  crate at all, even though the core is portable.
* Bad, because it discards portable functionality for no benefit, since nothing
  requires the build to fail on non-Unix.

### Fully support Windows

* Good, because Windows users would get a fully working, tested tool.
* Bad, because it adds shipping, CI, and support-burden obligations with no
  current demand.
* Bad, because it requires a Windows-appropriate alternative for every Unix-only
  behaviour (e.g. the executable bit), which has no natural equivalent.

## More Information

This stance is first exercised by the `add-executable-flag` change, whose
Unix-only `--executable` flag is the reason the question arose. Related:
ADR 0009 (Error vs warning policy) governs how a graceful non-Unix failure is
surfaced. Revisit this decision if real demand for Windows users emerges.
