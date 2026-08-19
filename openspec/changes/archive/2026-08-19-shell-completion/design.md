## Context

The CLI currently has a single flat `Args` struct parsed with `clap::derive`. It has no subcommands — `url` and `pattern` are required positional arguments. Adding a `completion` subcommand means restructuring the top-level parser to dispatch between the download workflow and the completion workflow.

## Goals / Non-Goals

**Goals:**
- Add a `completion <shell>` subcommand that prints a completion script to stdout and exits
- Reuse clap's built-in `clap_complete` integration — no hand-written completion logic
- Keep the existing download workflow unchanged

**Non-Goals:**
- Installing completion scripts automatically into the user's shell config
- Generating completion scripts at build time (on-demand via the subcommand is sufficient)

## Decisions

### Restructure top-level parser to use subcommands

**Decision:** Wrap the existing `Args` fields into a `Download` subcommand variant and add a `Completion` subcommand variant under a new top-level `Cli` enum.

**Rationale:** The `completion` subcommand must not require `url` and `pattern`, which are currently mandatory positional args on `Args`. The cleanest way to achieve this with clap's derive API is to introduce a `#[derive(Subcommand)]` enum. The alternative — a top-level `--generate-completion` flag with optional positionals — would require manual short-circuit logic before validation and is harder to document clearly.

**Alternative considered:** Keep the flat `Args` struct and add `--generate-completion <SHELL>` as an optional flag, making `url` and `pattern` optional. Rejected because it complicates validation (currently `url`/`pattern` are always required) and produces a less intuitive `--help` layout.

### Use `clap_complete` crate for script generation

**Decision:** Add `clap_complete` as a runtime dependency and call `clap_complete::generate()` with a `clap_complete::Shell` value parsed from the subcommand argument.

**Rationale:** `clap_complete` is the standard companion crate for clap v4 and handles all supported shells. `Shell` implements `clap::ValueEnum`, so argument parsing and error messages for unrecognized shell names are handled automatically by clap at no extra cost.

**Alternative considered:** `clap_complete_command` (a thin wrapper). Not necessary — the raw `clap_complete` API is straightforward enough.

### Exit immediately after printing completion script

**Decision:** In `main` (or `run`), match on the subcommand enum: if `Completion`, call `generate()` and return `Ok(())` before any network or file I/O.

**Rationale:** Keeps the download path completely unchanged and makes the short-circuit explicit and easy to test.

## Risks / Trade-offs

- **Subcommand restructure is a breaking CLI change** → Users invoking the binary directly (not via a subcommand) will need to prefix with the download subcommand name (e.g., `ghrls download <url> <pattern>`). Mitigated by choosing a clear, obvious subcommand name and updating documentation.
- **`clap_complete` output quality** → Script correctness depends on the upstream crate; no custom completion logic is involved, so there is nothing to maintain.
