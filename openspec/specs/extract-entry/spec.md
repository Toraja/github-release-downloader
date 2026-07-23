## Requirements

### Requirement: Flag definition
The CLI SHALL expose a `--extract-entry` flag (short: `-X`) that accepts a single string argument representing a path within the archive (e.g. `bin/mytool` or `share/config`).

#### Scenario: Flag accepted with a value
- **WHEN** the user passes `--extract-entry bin/mytool`
- **THEN** the CLI parses successfully and sets the extract-entry path to `bin/mytool`

#### Scenario: Flag rejected without a value
- **WHEN** the user passes `--extract-entry` with no argument
- **THEN** the CLI exits with a parse error

### Requirement: Mutual exclusion with --extract
`--extract-entry` SHALL be mutually exclusive with `--extract`. Passing both flags in the same invocation MUST produce a parse-time error.

#### Scenario: Both flags provided
- **WHEN** the user passes both `--extract-entry bin/mytool` and `--extract`
- **THEN** the CLI exits with a clap conflict error before any network request is made

### Requirement: Archive format validation
`--extract-entry` SHALL only work with `.tar.gz` and `.tgz` archives. If the matched asset has an unsupported format, the command MUST exit with a non-zero status and a descriptive error message.

#### Scenario: Supported format
- **WHEN** the matched asset name ends with `.tar.gz` or `.tgz`
- **THEN** extraction proceeds normally

#### Scenario: Unsupported format
- **WHEN** the matched asset name does not end with `.tar.gz` or `.tgz`
- **THEN** the CLI exits with exit code 1 and prints an error referencing the unsupported format

### Requirement: File entry extraction
When `--extract-entry` specifies a path that matches exactly one file entry in the archive (after normalising leading `./`), the CLI SHALL extract that single file to the resolved destination path.

#### Scenario: File entry extracted to default destination
- **WHEN** `--extract-entry bin/mytool` is given with no `--output` or `--dir`
- **THEN** the file is written to `./mytool` (basename of the entry in the current directory)

#### Scenario: File entry extracted to --dir destination
- **WHEN** `--extract-entry bin/mytool` and `--dir /opt/tools` are given
- **THEN** the file is written to `/opt/tools/mytool`

#### Scenario: File entry extracted and renamed via --output
- **WHEN** `--extract-entry bin/mytool` and `--output /usr/local/bin/tool` are given
- **THEN** the file is written to `/usr/local/bin/tool`

### Requirement: Directory entry extraction
When `--extract-entry` specifies a path that matches one or more archive entries sharing that path as a prefix (i.e. the entry is a directory), the CLI SHALL extract all entries under that prefix to the resolved destination, recreating the relative structure beneath it.

#### Scenario: Directory entry extracted to default destination
- **WHEN** `--extract-entry share/config` is given with no `--output` or `--dir`
- **THEN** a directory `./config/` is created containing all files from `share/config/` in the archive

#### Scenario: Directory entry extracted to --dir destination
- **WHEN** `--extract-entry share/config` and `--dir /etc/app` are given
- **THEN** `/etc/app/config/` is created with the directory's contents

#### Scenario: Directory entry extracted and renamed via --output
- **WHEN** `--extract-entry share/config` and `--output /etc/myapp` are given
- **THEN** `/etc/myapp/` is created with the directory's contents (root renamed)

### Requirement: Entry not found error
When the value of `--extract-entry` does not match any file or directory in the archive, the CLI MUST exit with a non-zero status and print an error. The error message SHALL include a list of top-level entries in the archive to help the user identify the correct path.

#### Scenario: Entry path does not exist in archive
- **WHEN** `--extract-entry no/such/path` is given
- **THEN** the CLI exits with exit code 1, prints "not found" (or equivalent), and lists available top-level entries

### Requirement: Parent directory creation
The CLI SHALL create any missing parent directories for the destination path automatically, consistent with the behaviour of `--output` and `--dir` in download mode.

#### Scenario: Destination parent does not exist
- **WHEN** `--extract-entry bin/mytool` and `--output /tmp/new/dir/tool` are given and `/tmp/new/dir/` does not exist
- **THEN** `/tmp/new/dir/` is created and the file is written to `/tmp/new/dir/tool`

### Requirement: Specified entry is a symlink
When the path given to `--extract-entry` resolves to a symlink entry in the archive, the CLI MUST exit with a non-zero status and a descriptive error. Symlink resolution is not supported.

#### Scenario: Directly specified entry is a symlink
- **WHEN** `--extract-entry bin/mytool` is given and `bin/mytool` is a symlink entry in the archive
- **THEN** the CLI exits with exit code 1 and prints an error stating that the entry is a symlink and is not supported

### Requirement: Child symlink entries skipped during directory extraction
When `--extract-entry` targets a directory and that directory contains symlink entries, the CLI SHALL skip those symlink entries, print a warning to stderr for each one, and continue extracting all regular file entries. This prevents path-traversal or unexpected filesystem side-effects.

#### Scenario: Archive contains a symlink under the target directory entry
- **WHEN** `--extract-entry bin/` targets a directory that contains a symlink entry among its children
- **THEN** the symlink entry is not extracted, a warning is printed to stderr, and all regular file entries under `bin/` are extracted normally

### Requirement: Directory entry extraction into existing destination
When `--extract-entry` targets a directory and the resolved destination already exists as a directory, the CLI SHALL merge the archive contents into that directory. Files present in the archive overwrite their counterparts in the destination; files already in the destination that are not in the archive are left untouched.

#### Scenario: Archive files overwrite existing destination files; unrelated files are preserved
- **WHEN** `--extract-entry mydir` is given, the resolved destination `./mydir/` already exists and contains `foo/bar` and `foo/quux`, and the archive contains `mydir/foo/bar` and `mydir/foo/baz`
- **THEN** extraction succeeds, `foo/bar` is overwritten with the archive version, `foo/baz` is newly created, and `foo/quux` is left untouched

### Requirement: Archive not saved to disk
When `--extract-entry` is used, the archive MUST NOT be written to disk; it SHALL be streamed and unpacked in-memory, consistent with the behaviour of `--extract`.

#### Scenario: Extraction leaves no archive file
- **WHEN** `--extract-entry bin/mytool` completes successfully
- **THEN** no `.tar.gz` or `.tgz` file is present in the working directory or destination directory
