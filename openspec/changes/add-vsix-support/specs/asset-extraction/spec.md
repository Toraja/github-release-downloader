## MODIFIED Requirements

### Requirement: Extract supported archive format when --extract is specified
When the `--extract` flag is provided, the CLI SHALL route the archive by format: for `.tar.gz`/`.tgz`, it SHALL stream the archive directly into a tar extractor; for zip-format assets (`.zip` or renamed zips such as `.vsix`), it SHALL buffer the archive fully in memory (zip requires seek) and extract it through the zip extractor. In both cases the archive is never saved to disk. When `--archive-entry` is provided, extraction narrows to that single entry — same format-routing applies.

#### Scenario: --extract with a tar.gz asset
- **WHEN** `--extract` and `--dir` are provided (without `--archive-entry`) and the asset ends in `.tar.gz` or `.tgz`
- **THEN** the whole archive is streamed and extracted into the specified directory and the CLI exits with code 0

#### Scenario: --extract narrowed by --archive-entry on a tar.gz asset
- **WHEN** `--extract --archive-entry PATH` on a `.tar.gz` asset
- **THEN** only the entry `PATH` is extracted (per `extract-entry`)

#### Scenario: --extract with a zip-format asset
- **WHEN** `--extract` and `--dir` are provided and the asset ends in `.zip` or `.vsix`
- **THEN** the archive is buffered fully in memory and unpacked via the zip extractor; it is never written to disk and the CLI exits with code 0

#### Scenario: Unsupported archive format with --extract
- **WHEN** `--extract` is provided but the asset filename matches neither supported tar.gz/tgz nor zip format registered in the CLI
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any download request

#### Scenario: Extraction failure
- **WHEN** extraction fails (e.g. corrupt archive, I/O error mid-stream)
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr