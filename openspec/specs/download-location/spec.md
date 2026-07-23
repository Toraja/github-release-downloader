## Purpose

Specifies how the CLI determines the location at which downloaded assets are saved, via optional `--dir` and `--output` flags.

## Requirements

### Requirement: Accept optional --dir flag for destination directory
The CLI SHALL accept an optional `--dir` / `-D` flag specifying the directory in which to save the downloaded asset.

#### Scenario: --dir flag omitted
- **WHEN** the `--dir` flag is not provided and `--output` is also not provided
- **THEN** the asset is saved to the current working directory with its original filename

#### Scenario: Existing directory given
- **WHEN** the `--dir` value is an existing directory path
- **THEN** the asset is saved inside that directory using the asset's original filename

#### Scenario: Non-existing directory given
- **WHEN** the `--dir` value is a directory path that does not yet exist
- **THEN** the directory and all missing parent directories are created automatically and the asset is saved inside using its original filename

### Requirement: Accept optional --output flag for exact file path
The CLI SHALL accept an optional `--output` / `-O` flag specifying the exact file path at which to save the downloaded asset, enabling the file to be renamed.

#### Scenario: Exact file path given
- **WHEN** the `--output` value is a path that does not exist or is an existing file
- **THEN** the asset is saved at that exact path, overwriting any existing file

#### Scenario: Output path is an existing directory
- **WHEN** the `--output` value is a path that resolves to an existing directory
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any HTTP request

#### Scenario: Parent directories do not exist
- **WHEN** the `--output` value references a directory hierarchy that does not fully exist
- **THEN** all missing parent directories are created automatically before the file is written

### Requirement: --dir and --output are mutually exclusive
The CLI SHALL reject invocations where both `--dir` and `--output` are provided simultaneously.

#### Scenario: Both flags provided
- **WHEN** the user provides both `--dir` and `--output` in the same invocation
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any HTTP request

### Requirement: --output and --extract are mutually exclusive
The CLI SHALL reject invocations where both `--output` and `--extract` are provided simultaneously, as `--output` specifies a single file path while `--extract` produces a directory of files.

#### Scenario: Both --output and --extract provided
- **WHEN** the user provides both `--output` and `--extract` in the same invocation
- **THEN** the CLI exits with a non-zero code and prints an error message to stderr before making any HTTP request
