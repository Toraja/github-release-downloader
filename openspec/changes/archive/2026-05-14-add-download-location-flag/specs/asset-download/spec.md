## MODIFIED Requirements

### Requirement: Download matched asset to current directory
The CLI SHALL download the matched asset binary and save it to the location determined by the `--dir` or `--output` flag, defaulting to the current working directory using the asset's original filename when neither flag is provided.

#### Scenario: Successful download with no output flags
- **WHEN** the asset download request succeeds and neither `--dir` nor `--output` is provided
- **THEN** the file is written to the current working directory with the asset's original name and the CLI exits with code 0

#### Scenario: Successful download with --dir
- **WHEN** the asset download request succeeds and `--dir` is provided
- **THEN** the file is written to that directory with the asset's original name and the CLI exits with code 0

#### Scenario: Successful download with --output
- **WHEN** the asset download request succeeds and `--output` is provided
- **THEN** the file is written to that exact path and the CLI exits with code 0

#### Scenario: Download request failure
- **WHEN** the asset download HTTP request fails
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr
