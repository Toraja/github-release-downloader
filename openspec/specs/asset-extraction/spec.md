## Purpose

Specifies how the CLI handles extraction of downloaded archive assets when the `--extract` flag is provided.

## Requirements

### Requirement: Extract .tar.gz asset when --extract is specified
When the `--extract` flag is provided, the CLI SHALL stream the downloaded asset directly into a tar extractor and unpack it to the destination directory, then exit with code 0. The archive is not saved to disk.

#### Scenario: --extract with --dir
- **WHEN** `--extract` and `--dir` are provided and the asset is a `.tar.gz` or `.tgz` file
- **THEN** the archive is streamed and extracted into the specified directory and the CLI exits with code 0

#### Scenario: --extract with no location flags
- **WHEN** `--extract` is provided without `--dir` or `--output` and the asset is a `.tar.gz` or `.tgz` file
- **THEN** the archive is streamed and extracted into the current working directory and the CLI exits with code 0

#### Scenario: Unsupported archive format with --extract
- **WHEN** `--extract` is provided but the asset filename does not end in `.tar.gz` or `.tgz`
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any download request

#### Scenario: Extraction failure
- **WHEN** `--extract` is provided but extraction fails (e.g. corrupt archive, I/O error mid-stream)
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr

### Requirement: Print destination directory on successful extraction
When `--extract` is used and extraction succeeds, the CLI SHALL print `Extracted to: <dir>` to stdout, where `<dir>` is the destination directory.

#### Scenario: Extraction succeeds with --dir
- **WHEN** extraction completes successfully and `--dir` was provided
- **THEN** the CLI prints `Extracted to: <dir>` using the resolved directory path

#### Scenario: Extraction succeeds with no location flags
- **WHEN** extraction completes successfully and neither `--dir` nor `--output` was provided
- **THEN** the CLI prints `Extracted to: .` (or the resolved current directory path)
