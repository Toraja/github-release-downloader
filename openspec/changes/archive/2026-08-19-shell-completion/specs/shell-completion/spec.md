## Purpose

Provides a `completion` subcommand that generates and prints a shell completion script for the specified shell, enabling tab-completion of the CLI's flags and arguments.

## ADDED Requirements

### Requirement: Completion subcommand exists
The CLI SHALL expose a `completion` subcommand that accepts a single positional argument identifying the target shell.

#### Scenario: Subcommand is invoked with a valid shell
- **WHEN** the user runs `ghrls completion <shell>`
- **THEN** the CLI SHALL print a completion script for that shell to stdout and exit with code 0

#### Scenario: Subcommand is invoked without a shell argument
- **WHEN** the user runs `ghrls completion` with no argument
- **THEN** the CLI SHALL exit with a non-zero code and print an error message indicating the shell argument is required

#### Scenario: Subcommand is invoked with an unrecognized shell name
- **WHEN** the user runs `ghrls completion <unknown>`
- **THEN** the CLI SHALL exit with a non-zero code and print an error listing the supported shells

### Requirement: Supported shells
The `completion` subcommand SHALL support at least the following shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

#### Scenario: Each supported shell produces a non-empty script
- **WHEN** the user runs `ghrls completion <shell>` for any supported shell value
- **THEN** the CLI SHALL print a non-empty completion script to stdout

### Requirement: No download performed during completion generation
When the `completion` subcommand is used, the CLI SHALL NOT perform any network requests or file downloads.

#### Scenario: Completion does not require a URL or pattern argument
- **WHEN** the user runs `ghrls completion <shell>`
- **THEN** the CLI SHALL succeed without requiring the `url` or `pattern` positional arguments that are mandatory for the download workflow
