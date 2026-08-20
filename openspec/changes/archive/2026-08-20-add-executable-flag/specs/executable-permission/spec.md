## Purpose

Defines the `--executable` flag, which sets the executable bit on the file the CLI produces so that downloaded or extracted binaries are ready to run without a manual `chmod`.

## ADDED Requirements

### Requirement: Flag definition
The `download` subcommand SHALL expose an `--executable` flag (short: `-e`) that takes no value. When present, the CLI SHALL add executable permission to the resulting target after the asset is written to disk. The flag defaults to off.

#### Scenario: Flag accepted as a boolean
- **WHEN** the user passes `--executable`
- **THEN** the CLI parses successfully and enables adding executable permission

#### Scenario: Flag rejects a value
- **WHEN** the user passes `--executable somevalue`
- **THEN** `somevalue` is treated as a separate argument (the flag itself consumes nothing)

#### Scenario: Flag omitted leaves permissions unchanged
- **WHEN** `--executable` is not provided
- **THEN** the CLI does not modify the permission bits of the produced file

### Requirement: Permission semantics
When `--executable` is applied, the CLI SHALL OR the executable bits `0o111` onto the target's current mode, granting execute permission to user, group, and other regardless of the process umask. All other permission bits present on the target SHALL be preserved.

#### Scenario: Read-only file gains execute for all
- **WHEN** the target file has mode `0o644` and `--executable` is applied
- **THEN** the resulting mode is `0o755`

#### Scenario: Existing bits preserved
- **WHEN** the target file has mode `0o600` and `--executable` is applied
- **THEN** the resulting mode is `0o711` (existing owner read/write preserved, execute added for all)

#### Scenario: Already-executable file is unchanged
- **WHEN** the target file already has mode `0o755` and `--executable` is applied
- **THEN** the resulting mode remains `0o755`

### Requirement: Target in plain download mode
When `--executable` is used without `--extract`, the CLI SHALL apply executable permission to the downloaded asset file at its resolved destination (default directory, `--dir`, or `--output`).

#### Scenario: Downloaded file made executable
- **WHEN** the user passes `--executable` without `--extract`
- **THEN** the downloaded asset file receives executable permission at its final path

### Requirement: Target in single-entry extraction mode
When `--executable` is used together with `--extract` and `--archive-entry`, and the entry resolves to a single file, the CLI SHALL apply executable permission to that extracted file at its resolved destination (default, `--dir`, or `--output`), regardless of which location flag determined the path.

#### Scenario: Extracted file entry made executable
- **WHEN** the user passes `--extract --archive-entry bin/mytool --executable`
- **THEN** the extracted file receives executable permission at its resolved path

#### Scenario: Extracted and renamed file made executable
- **WHEN** the user passes `--extract --archive-entry bin/mytool --output /usr/local/bin/tool --executable`
- **THEN** the file written to `/usr/local/bin/tool` receives executable permission

### Requirement: Whole-archive extraction rejected
`--executable` combined with `--extract` but without `--archive-entry` SHALL be rejected, because whole-archive extraction produces a directory of files with no single unambiguous target. The CLI MUST detect this before making any network request and exit with a non-zero status and a descriptive error.

#### Scenario: Executable with whole-archive extraction is rejected
- **WHEN** the user passes `--extract --executable` without `--archive-entry`
- **THEN** the CLI exits non-zero with an argument-conflict error before making any HTTP request, directing the user to narrow extraction with `--archive-entry`

### Requirement: Directory target is a user error
When `--executable` is used with `--extract --archive-entry` and the entry resolves to a directory, the CLI SHALL treat this as user misuse. Because directory-ness is only known after extraction, the CLI SHALL complete extraction, print the extraction success line, then print an error stating that `--executable` requires a file target, apply no permission change, and exit with a non-zero status.

#### Scenario: Directory entry with --executable errors after extraction
- **WHEN** the user passes `--extract --archive-entry share/config --executable` and `share/config` is a directory in the archive
- **THEN** the directory is extracted, the extraction success line is printed, an error stating `--executable` requires a file target is printed to stderr, no permission bits are changed, and the CLI exits non-zero

### Requirement: Permission-set failure is partial success
When setting the executable bit fails after the asset has already been written or extracted, the CLI SHALL report the failure and exit with a non-zero status, but the successfully written asset SHALL remain on disk. The output SHALL make clear that the download or extraction itself succeeded so the user does not repeat it.

#### Scenario: chmod failure after successful download
- **WHEN** the asset is written successfully but the CLI cannot set its executable bit
- **THEN** the success line for the download or extraction is printed, an error describing the permission failure is printed to stderr, the asset remains on disk, and the CLI exits non-zero

### Requirement: Platform support
Setting the executable bit is a Unix-only operation. On Unix targets (including Linux and macOS) `--executable` SHALL function as specified. Non-Unix targets are not supported for this behavior.

#### Scenario: Executable bit set on a Unix target
- **WHEN** the CLI runs on a Unix platform and `--executable` is applied to a file target
- **THEN** the executable bits are set on that file
