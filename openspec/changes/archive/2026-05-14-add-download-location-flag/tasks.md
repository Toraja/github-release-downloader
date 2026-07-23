## 1. CLI Arguments

- [x] 1.1 Add `--dir` / `-D` optional flag to the `Cli` struct using clap, accepting an `Option<PathBuf>`
- [x] 1.2 Add `--output` / `-O` optional flag to the `Cli` struct using clap, accepting an `Option<PathBuf>`
- [x] 1.3 Mark `--dir` and `--output` as mutually exclusive using clap's `conflicts_with`
- [x] 1.4 Add help text for `--dir` (destination directory, original filename preserved)
- [x] 1.5 Add help text for `--output` (exact file path, enables rename; error if path is an existing directory)

## 2. Output Path Resolution

- [x] 2.1 Implement a helper function that resolves the final file path:
  - `--dir` given → join directory with asset's original filename
  - `--output` given and path is an existing directory → return an error
  - `--output` given and path is a file or does not exist → use path as-is
  - neither flag → join current working directory with asset's original filename

## 3. Directory Creation

- [x] 3.1 Before writing the file, call `fs::create_dir_all` on the parent directory of the resolved output path

## 4. File Write Integration

- [x] 4.1 Thread the resolved output path through to the file-write step, replacing the current hard-coded current-directory logic

## 5. Tests

- [x] 5.1 Test path resolution: `--dir` with existing directory → original filename appended
- [x] 5.2 Test path resolution: `--dir` with non-existing directory → directory created, original filename appended
- [x] 5.3 Test path resolution: `--output` with non-existing path → used as-is
- [x] 5.4 Test path resolution: `--output` with existing file path → used as-is (overwrite)
- [x] 5.5 Test path resolution: `--output` with existing directory path → error returned
- [x] 5.6 Test path resolution: neither flag → current working directory with original filename
- [x] 5.7 Test that providing both `--dir` and `--output` is rejected by clap
