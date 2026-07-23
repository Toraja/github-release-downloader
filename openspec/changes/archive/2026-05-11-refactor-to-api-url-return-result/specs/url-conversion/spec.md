## MODIFIED Requirements

### Requirement: Convert GitHub repo URL to API latest release URL
The CLI SHALL convert a GitHub repo URL of the form `https://github.com/<owner>/<repo>` to a `url::Url` representing `https://api.github.com/repos/<owner>/<repo>/releases/latest`. The internal `to_api_url` function SHALL return `Result<url::Url, String>` instead of `Result<String, String>`.

#### Scenario: Standard URL without trailing slash
- **WHEN** the input is `https://github.com/owner/repo`
- **THEN** the output is a `url::Url` equal to `https://api.github.com/repos/owner/repo/releases/latest`

#### Scenario: URL with trailing slash
- **WHEN** the input is `https://github.com/owner/repo/`
- **THEN** the output is a `url::Url` equal to `https://api.github.com/repos/owner/repo/releases/latest` (trailing slash ignored)
