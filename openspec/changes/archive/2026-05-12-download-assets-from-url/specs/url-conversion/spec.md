## MODIFIED Requirements

### Requirement: Convert GitHub repo URL to API latest release URL
The CLI SHALL convert a GitHub repo URL of the form `https://github.com/<owner>/<repo>` to a `url::Url` representing `https://api.github.com/repos/<owner>/<repo>/releases/latest`. This URL is used internally to fetch release metadata and is no longer printed to stdout.

#### Scenario: Standard URL without trailing slash
- **WHEN** the input is `https://github.com/owner/repo`
- **THEN** the derived API URL is `https://api.github.com/repos/owner/repo/releases/latest`

#### Scenario: URL with trailing slash
- **WHEN** the input is `https://github.com/owner/repo/`
- **THEN** the derived API URL is `https://api.github.com/repos/owner/repo/releases/latest` (trailing slash ignored)
