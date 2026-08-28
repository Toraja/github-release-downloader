# Release Tag Specification

## Purpose

Provide an automation-friendly way to retrieve the latest GitHub release tag while optionally removing a repository-specific leading prefix.

## Requirements

### Requirement: Retrieve the latest release tag
The CLI SHALL accept `tag <URL>`, fetch the repository's latest release metadata from GitHub, and print its tag exactly as returned followed by a newline. The CLI SHALL treat the tag as an opaque string without semantic-version validation and SHALL NOT download any release asset.

#### Scenario: Latest release has a semantic-version tag
- **WHEN** the user runs `ghrls tag https://github.com/owner/repo` and the latest release tag is `v1.2.3`
- **THEN** the CLI prints `v1.2.3` followed by a newline to stdout and exits successfully

#### Scenario: Latest release has a non-semantic tag
- **WHEN** the user runs the `tag` subcommand and the latest release tag is `stable-2026-08`
- **THEN** the CLI prints `stable-2026-08` followed by a newline without rejecting or transforming it

#### Scenario: Latest release request fails
- **WHEN** the GitHub latest-release request fails or its response cannot be parsed
- **THEN** the CLI exits with a non-zero status and reports the error to stderr

### Requirement: Authenticate the latest release request
The `tag` subcommand SHALL use the `GITHUB_TOKEN` environment variable for GitHub API authentication when it is set.

#### Scenario: GitHub token is set
- **WHEN** the user runs the `tag` subcommand with `GITHUB_TOKEN` set
- **THEN** the latest-release request includes the token as bearer authentication

### Requirement: Optionally strip a literal tag prefix
The CLI SHALL accept `--strip-prefix <PREFIX>` on the `tag` subcommand. When the tag begins with the given prefix, the CLI SHALL remove exactly one occurrence from the start before printing it. Matching SHALL be literal and case-sensitive; when the prefix is absent, the tag SHALL remain unchanged.

#### Scenario: Tag begins with the requested prefix
- **WHEN** the latest release tag is `v1.2.3` and the user supplies `--strip-prefix v`
- **THEN** the CLI prints `1.2.3` followed by a newline

#### Scenario: Prefix occurs more than once at the start
- **WHEN** the latest release tag is `vv1.2.3` and the user supplies `--strip-prefix v`
- **THEN** the CLI prints `v1.2.3` followed by a newline

#### Scenario: Tag does not begin with the requested prefix
- **WHEN** the latest release tag is `1.2.3` and the user supplies `--strip-prefix v`
- **THEN** the CLI prints `1.2.3` followed by a newline

#### Scenario: Prefix differs only by case
- **WHEN** the latest release tag is `V1.2.3` and the user supplies `--strip-prefix v`
- **THEN** the CLI prints `V1.2.3` followed by a newline

#### Scenario: Prefix equals the complete tag
- **WHEN** the latest release tag is `v` and the user supplies `--strip-prefix v`
- **THEN** the CLI prints only a newline and exits successfully
