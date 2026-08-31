## MODIFIED Requirements

### Requirement: Archive format validation
`--archive-entry` SHALL only work with `.tar.gz`, `.tgz`, and `.zip` archives. If the matched asset has an unsupported format, the command MUST exit with a non-zero status and a descriptive error message.

#### Scenario: Supported format
- **WHEN** the matched asset name ends with `.tar.gz`, `.tgz`, or `.zip`
- **THEN** extraction proceeds normally

#### Scenario: Unsupported format
- **WHEN** the matched asset name does not end with `.tar.gz`, `.tgz`, or `.zip`
- **THEN** the CLI exits with exit code 1 and prints an error referencing the unsupported format

### Requirement: Archive not saved to disk
When `--archive-entry` is used, the archive MUST NOT be written to disk; it SHALL be unpacked from the downloaded response body without persisting the archive file, consistent with the behaviour of `--extract`.

#### Scenario: Extraction leaves no archive file
- **WHEN** `--extract --archive-entry bin/mytool` completes successfully
- **THEN** no `.tar.gz`, `.tgz`, or `.zip` file is present in the working directory or destination directory
