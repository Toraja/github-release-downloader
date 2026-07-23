## 1. CLI Flag

- [x] 1.1 Add `extract_entry: Option<String>` field to `Cli` with short flag `-X`, long `--extract-entry`, and `conflicts_with = "extract"`

## 2. Destination Resolution

- [x] 2.1 Add `resolve_entry_dest` function that computes the destination path for `--extract-entry` (no `is_dir` guard — existing directory is valid when the entry is a directory)

## 3. Core Extraction Logic

- [x] 3.1 Add `normalize_entry_path` helper that strips a leading `./` and a trailing `/` from an archive entry path
- [x] 3.2 Add `extract_entry` function that streams the archive, normalises paths, matches a file entry (exact match) or directory entry (prefix match), and extracts to the resolved destination
- [x] 3.3 Handle entry-not-found in `extract_entry`: exit with error and include a list of top-level archive entries in the message
- [x] 3.4 Handle symlinks in `extract_entry`: if the specified entry itself is a symlink, return an error; if a child entry during directory extraction is a symlink, print a warning to stderr and skip it

## 4. Integration in main

- [x] 4.1 In `main`, after asset selection, check `cli.extract_entry`; validate format with `is_extractable`, resolve destination with `resolve_entry_dest`, and call `extract_entry`

## 5. Tests

- [x] 5.1 `--extract-entry` and `--extract` are mutually exclusive (clap parse-time error)
- [x] 5.2 File entry extracted to default destination (basename in current directory)
- [x] 5.3 File entry extracted with `--dir` (basename placed under the given directory)
- [x] 5.4 File entry extracted with `--output` (written to exact path, effectively renaming it)
- [x] 5.5 Directory entry extracted to default destination (basename directory created in current directory)
- [x] 5.6 Directory entry extracted with `--dir`
- [x] 5.7 Directory entry extracted with `--output` (root directory renamed to the given path)
- [x] 5.8 Entry not found returns error that lists available top-level entries
- [x] 5.9 Unsupported archive format with `--extract-entry` returns a non-zero error
- [x] 5.10 Directly specified entry that is a symlink returns an error
- [x] 5.11 Child symlink entries during directory extraction are skipped with a stderr warning; regular files are still extracted
- [x] 5.12 Missing parent directories of the destination are created automatically
