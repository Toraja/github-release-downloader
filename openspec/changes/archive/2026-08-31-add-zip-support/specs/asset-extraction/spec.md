## MODIFIED Requirements

### Requirement: Extract archvie asset when --extract is specified
When the `--extract` flag is provided without `--archive-entry`, the CLI SHALL unpack the **entire** archive to the destination directory, then exit with code 0. The archive is not saved to disk. When `--archive-entry` is also provided, extraction is narrowed to that single entry (see the `extract-entry` capability); the whole-archive behaviour applies only in the absence of `--archive-entry`. Supported archive formats are `.tar.gz`, `.tgz`, and `.zip`.

#### Scenario: --extract with --dir
- **WHEN** `--extract` and `--dir` are provided (without `--archive-entry`) and the asset is a `.tar.gz`, `.tgz`, or `.zip` file
- **THEN** the whole archive is extracted into the specified directory and the CLI exits with code 0

#### Scenario: --extract with no location flags
- **WHEN** `--extract` is provided without `--dir`, `--output`, or `--archive-entry` and the asset is a `.tar.gz`, `.tgz`, or `.zip` file
- **THEN** the whole archive is extracted into the current working directory and the CLI exits with code 0

#### Scenario: --extract narrowed by --archive-entry
- **WHEN** `--extract` and `--archive-entry PATH` are provided and the asset is a `.tar.gz`, `.tgz`, or `.zip` file
- **THEN** only the entry `PATH` is extracted (per the `extract-entry` capability) rather than the whole archive

#### Scenario: Unsupported archive format with --extract
- **WHEN** `--extract` is provided but the asset filename does not end in `.tar.gz`, `.tgz`, or `.zip`
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any download request

#### Scenario: Extraction failure
- **WHEN** `--extract` is provided but extraction fails (e.g. corrupt archive, I/O error)
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr
