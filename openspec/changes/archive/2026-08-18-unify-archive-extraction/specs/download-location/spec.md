## REMOVED Requirements

### Requirement: --output and --extract are mutually exclusive
**Reason**: The conflict was unconditional, which prevented renaming a single extracted entry. It is replaced by a conditional rule (see the new "--output is conditionally compatible with --extract" requirement) that only rejects `--output` for whole-archive extraction.
**Migration**: `--extract --output PATH` (whole archive) is still rejected — use `--dir` instead. To rename a single extracted entry, combine `--extract --archive-entry ENTRY --output PATH`, which is now accepted.

## ADDED Requirements

### Requirement: --output is conditionally compatible with --extract
The CLI SHALL reject `--output` together with `--extract` only when `--archive-entry` is absent, because whole-archive extraction produces a directory of files rather than a single renameable path. When `--archive-entry` is present, `--output` renames the single extracted entry and MUST be accepted. Because this conditional constraint cannot be expressed with clap's declarative conflict/requirement attributes, the CLI SHALL enforce it via a post-parse validation that emits a clap-style argument-conflict error with a message directing the user to `--dir`.

#### Scenario: --output with whole-archive extraction is rejected
- **WHEN** the user provides `--extract` and `--output` without `--archive-entry`
- **THEN** the CLI exits with a non-zero code and prints an argument-conflict error to stderr before making any HTTP request

#### Scenario: --output with single-entry extraction is accepted
- **WHEN** the user provides `--extract`, `--archive-entry ENTRY`, and `--output PATH`
- **THEN** the CLI parses successfully and the extracted entry is written to `PATH`

#### Scenario: --output with a plain download is unaffected
- **WHEN** the user provides `--output` without `--extract`
- **THEN** the CLI parses successfully and the downloaded asset is written to the given path
