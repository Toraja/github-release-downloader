## ADDED Requirements

### Requirement: Accept asset name regex as argument
The CLI SHALL accept a second positional argument: a regex pattern used to filter release assets by name.

#### Scenario: Valid regex provided
- **WHEN** user runs the CLI with a valid GitHub repo URL and a valid regex pattern
- **THEN** the CLI proceeds to fetch release metadata and filter assets using the pattern

#### Scenario: Invalid regex provided
- **WHEN** user provides a string that is not a valid regex pattern
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any HTTP request

### Requirement: Fetch latest release metadata
The CLI SHALL make an HTTP GET request to the GitHub API latest release endpoint derived from the input URL.

#### Scenario: Successful API response
- **WHEN** the GitHub API returns a valid JSON response for the latest release
- **THEN** the CLI parses the asset list from the response

#### Scenario: API request failure
- **WHEN** the HTTP request fails (e.g., network error, non-2xx response)
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr

#### Scenario: GITHUB_TOKEN set
- **WHEN** the `GITHUB_TOKEN` environment variable is set
- **THEN** the CLI includes an `Authorization: Bearer <token>` header in the API request

### Requirement: Match exactly one asset by regex
The CLI SHALL match asset names against the provided regex and enforce that exactly one asset matches.

#### Scenario: Exactly one asset matches
- **WHEN** exactly one asset name matches the regex
- **THEN** the CLI proceeds to download that asset

#### Scenario: No assets match
- **WHEN** no asset names match the regex
- **THEN** the CLI exits with a non-zero code, prints an error to stderr, and lists all available asset names

#### Scenario: Multiple assets match
- **WHEN** more than one asset name matches the regex
- **THEN** the CLI exits with a non-zero code, prints an error to stderr, and lists all matching asset names

### Requirement: Download matched asset to current directory
The CLI SHALL download the matched asset binary and save it to the current working directory using the asset's original filename.

#### Scenario: Successful download
- **WHEN** the asset download request succeeds
- **THEN** the file is written to the current working directory with the asset's original name and the CLI exits with code 0

#### Scenario: Download request failure
- **WHEN** the asset download HTTP request fails
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr
