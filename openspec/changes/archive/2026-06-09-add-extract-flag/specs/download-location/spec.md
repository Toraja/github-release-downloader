## ADDED Requirements

### Requirement: --output and --extract are mutually exclusive
The CLI SHALL reject invocations where both `--output` and `--extract` are provided simultaneously, as `--output` specifies a single file path while `--extract` produces a directory of files.

#### Scenario: Both --output and --extract provided
- **WHEN** the user provides both `--output` and `--extract` in the same invocation
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any HTTP request
