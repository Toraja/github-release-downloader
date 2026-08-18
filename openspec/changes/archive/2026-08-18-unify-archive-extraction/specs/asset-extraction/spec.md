## MODIFIED Requirements

### Requirement: Extract .tar.gz asset when --extract is specified
When the `--extract` flag is provided without `--archive-entry`, the CLI SHALL stream the downloaded asset directly into a tar extractor and unpack the **entire** archive to the destination directory, then exit with code 0. The archive is not saved to disk. When `--archive-entry` is also provided, extraction is narrowed to that single entry (see the `extract-entry` capability); the whole-archive behaviour applies only in the absence of `--archive-entry`.

#### Scenario: --extract with --dir
- **WHEN** `--extract` and `--dir` are provided (without `--archive-entry`) and the asset is a `.tar.gz` or `.tgz` file
- **THEN** the whole archive is streamed and extracted into the specified directory and the CLI exits with code 0

#### Scenario: --extract with no location flags
- **WHEN** `--extract` is provided without `--dir`, `--output`, or `--archive-entry` and the asset is a `.tar.gz` or `.tgz` file
- **THEN** the whole archive is streamed and extracted into the current working directory and the CLI exits with code 0

#### Scenario: --extract narrowed by --archive-entry
- **WHEN** `--extract` and `--archive-entry PATH` are provided and the asset is a `.tar.gz` or `.tgz` file
- **THEN** only the entry `PATH` is extracted (per the `extract-entry` capability) rather than the whole archive

#### Scenario: Unsupported archive format with --extract
- **WHEN** `--extract` is provided but the asset filename does not end in `.tar.gz` or `.tgz`
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any download request

#### Scenario: Extraction failure
- **WHEN** `--extract` is provided but extraction fails (e.g. corrupt archive, I/O error mid-stream)
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr

### Requirement: Print destination directory on successful extraction
When `--extract` is used and extraction succeeds, the CLI SHALL print `Extracted to: <dest>` to stdout, where `<dest>` is the resolved destination path (the destination directory for whole-archive extraction, or the resolved entry destination when `--archive-entry` is used).

#### Scenario: Extraction succeeds with --dir
- **WHEN** whole-archive extraction completes successfully and `--dir` was provided
- **THEN** the CLI prints `Extracted to: <dir>` using the resolved directory path

#### Scenario: Extraction succeeds with no location flags
- **WHEN** whole-archive extraction completes successfully and neither `--dir` nor `--output` was provided
- **THEN** the CLI prints `Extracted to: .` (or the resolved current directory path)

#### Scenario: Entry extraction succeeds
- **WHEN** extraction narrowed by `--archive-entry` completes successfully
- **THEN** the CLI prints `Extracted to: <dest>` using the resolved entry destination path
