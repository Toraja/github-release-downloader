## Purpose

A convenience subcommand that turns a live URL and asset regex into an installed VSIX extension required by non-vscode editors — one directory of extracted extension contents placed like VSCode would do — without requiring unzip and directory-shuffling by hand.

## ADDED Requirements

### Requirement: Match and validate a vsix asset before downloading
The CLI SHALL accept `vsix <URL> <PATTERN>`, match exactly one asset from the repo's latest release by regex, and verify the matched asset's name ends with `.vsix` before downloading. If no asset or multiple assets match, or if the matched asset does not end with `.vsix`, the CLI SHALL exit with a non-zero code and print an error to stderr *without making any download request*.

#### Scenario: Exact match, valid .vsix extension
- **WHEN** `ghrls vsix https://github.com/owner/repo 'codelldb-linux-x64\.vsix'` matches exactly one asset named `codelldb-linux-x64.vsix`
- **THEN** the CLI proceeds to download and extract that asset

#### Scenario: Zero or multiple matches
- **WHEN** the pattern matches no assets or multiple assets in the latest release
- **THEN** the CLI exits with a non-zero code and prints an error naming the matched/available assets — same behavior as `download`

#### Scenario: Asset is not a .vsix
- **WHEN** the matched asset is `tool.tar.gz` (or any non-`.vsix` name)
- **THEN** the CLI exits with a non-zero code and prints an error before downloading

### Requirement: Extract the vsix `extension` entry's contents — never save the archive to disk
The CLI SHALL stream the matched `.vsix` asset directly into a zip extractor and unpack the *contents of* the archive entry `extension/` into the destination — the `extension/` wrapper itself is stripped. The archive is fully buffered in memory and is never saved to disk. Vsix-specific metadata files (for example `extension.vsixmanifest`, `[Content_Types].xml`) SHALL be discarded; they are part of the vsix packaging format, not extension content.

#### Scenario: Extraction with --dir
- **WHEN** `ghrls vsix <URL> <PATTERN> --dir ~/vscode-exts/codelldb`
- **THEN** the contents of the archive entry `extension/` land directly in `~/vscode-exts/codelldb/` and the CLI exits with code 0

#### Scenario: Extraction without --dir
- **WHEN** `ghrls vsix <URL> <PATTERN>` without `--dir`
- **THEN** the contents of the archive entry `extension/` land directly in the current working directory and the CLI exits with code 0

#### Scenario: Meta-files discarded (vsix-specific artifacts)
- **WHEN** the archive contains `extension.vsixmanifest` or `[Content_Types].xml` at its root
- **THEN** those files are silently skipped and not extracted

#### Scenario: Extraction failure
- **WHEN** the download or extraction fails (e.g. corrupt zip, I/O error mid-stream)
- **THEN** the CLI exits with a non-zero code and prints an error to stderr

### Requirement: Print the destination directory on successful extraction
The CLI SHALL print the destination path followed by a newline after a successful extraction, in the same format as the `download` command prints "Extracted to: ...".

#### Scenario: Successful extraction prints destination
- **WHEN** `vsix` runs successfully with `--dir ~/vscode-exts/codelldb`
- **THEN** the CLI prints `Extracted to: ~/vscode-exts/codelldb` (or `Extracted to: <cwd>` without `--dir`)

## MODIFIED Requirements

(No modified requirements in this capability.)

## REMOVED Requirements

(No removed requirements in this capability.)
