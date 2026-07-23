### Requirement: Accept GitHub repo URL as input
The CLI SHALL accept a single GitHub repository URL as a positional argument.

#### Scenario: Valid URL provided
- **WHEN** user runs the CLI with a valid GitHub repo URL (e.g., `https://github.com/owner/repo`)
- **THEN** the CLI exits with code 0 and outputs the API URL to stdout

#### Scenario: No argument provided
- **WHEN** user runs the CLI with no arguments
- **THEN** the CLI exits with a non-zero code and prints a usage/help message to stderr

### Requirement: Convert GitHub repo URL to API latest release URL
The CLI SHALL convert a GitHub repo URL of the form `https://github.com/<owner>/<repo>` to a `url::Url` representing `https://api.github.com/repos/<owner>/<repo>/releases/latest`. This URL is used internally to fetch release metadata and is no longer printed to stdout.

#### Scenario: Standard URL without trailing slash
- **WHEN** the input is `https://github.com/owner/repo`
- **THEN** the derived API URL is `https://api.github.com/repos/owner/repo/releases/latest`

#### Scenario: URL with trailing slash
- **WHEN** the input is `https://github.com/owner/repo/`
- **THEN** the derived API URL is `https://api.github.com/repos/owner/repo/releases/latest` (trailing slash ignored)

### Requirement: Reject invalid or non-GitHub URLs
The CLI SHALL exit with a non-zero code and print an error message to stderr when the input URL is not a valid GitHub repo URL.

#### Scenario: Non-GitHub domain
- **WHEN** the input URL does not have `github.com` as the host
- **THEN** the CLI exits with a non-zero code and prints an error to stderr

#### Scenario: Missing repo segment
- **WHEN** the input URL is `https://github.com/owner` (no repo path)
- **THEN** the CLI exits with a non-zero code and prints an error to stderr
