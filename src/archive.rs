use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::error::AppError;

/// Describes where the extracted result should land.
///
/// - `Into(dir)`: place the naturally-named result *into* this directory
///   (whole archive unpacks directly into `dir`; a single entry lands at `dir/basename(entry)`).
/// - `Exact(path)`: use this path verbatim (renames the result).
///   Valid only when extracting a single entry; passing `Exact` for whole-archive
///   extraction is a programming error guarded by `Args::validate` in `main`.
#[derive(Debug)]
pub enum Destination {
    Into(PathBuf),
    Exact(PathBuf),
}

impl Destination {
    /// Resolve a `Destination` from two optional paths.
    /// `exact` takes precedence; falls back to `into`, defaulting to `.`.
    /// Returns an error if `exact` points to an existing directory.
    pub fn resolve(
        into: Option<&std::path::Path>,
        exact: Option<&std::path::Path>,
    ) -> Result<Self, crate::error::AppError> {
        match exact {
            Some(p) => {
                if p.is_dir() {
                    return Err(crate::error::AppError::OutputIsDir(p.display().to_string()));
                }
                Ok(Destination::Exact(p.to_path_buf()))
            }
            None => Ok(Destination::Into(
                into.unwrap_or(std::path::Path::new(".")).to_path_buf(),
            )),
        }
    }
}

pub fn is_extractable(asset_name: &str) -> bool {
    asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz")
}

/// Strip a leading `./` and a trailing `/` from an archive entry path.
fn normalize_entry_path(s: &str) -> &str {
    let s = s.strip_prefix("./").unwrap_or(s);
    s.trim_end_matches('/')
}

pub fn save_to_file(
    reader: impl Read,
    asset_name: &str,
    dest: Destination,
) -> Result<PathBuf, AppError> {
    let path = match dest {
        Destination::Into(dir) => dir.join(asset_name),
        Destination::Exact(p) => p,
    };

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    let mut file = File::create(&path).map_err(|e| AppError::CreateFile {
        path: path.display().to_string(),
        source: e,
    })?;

    let mut reader = reader;
    io::copy(&mut reader, &mut file).map_err(|e| AppError::WriteFile {
        path: path.display().to_string(),
        source: e,
    })?;

    Ok(path)
}

/// Unified extraction entry point.
///
/// - `entry = None`: extract the whole archive into the directory described by `dest`.
///   `Destination::Exact` is invalid here (guarded by `Args::validate`).
/// - `entry = Some(path)`: extract a single file or directory entry, landing it at
///   the path resolved from `dest`.
///
/// Returns the filesystem path of the extracted result.
pub fn extract_archive(
    reader: impl Read,
    entry: Option<&str>,
    dest: Destination,
) -> Result<PathBuf, AppError> {
    match entry {
        None => {
            // Whole-archive extraction. Destination::Exact is unreachable here because
            // Args::validate rejects --output without --archive-entry.
            let dir = match dest {
                Destination::Into(d) => d,
                Destination::Exact(_) => unreachable!(
                    "Destination::Exact is invalid for whole-archive extraction; \
                     Args::validate should have caught this"
                ),
            };
            fs::create_dir_all(&dir).map_err(|e| AppError::CreateDir {
                path: dir.display().to_string(),
                source: e,
            })?;
            unpack_tar_gz(reader, &dir)?;
            Ok(dir)
        }
        Some(entry_path) => {
            let norm = normalize_entry_path(entry_path);
            let landing = match dest {
                Destination::Exact(p) => p,
                Destination::Into(dir) => {
                    let basename = Path::new(norm)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or(norm);
                    dir.join(basename)
                }
            };
            extract_archive_entry(reader, entry_path, &landing)?;
            Ok(landing)
        }
    }
}

fn unpack_tar_gz<R: Read>(reader: R, dest_dir: &Path) -> Result<(), AppError> {
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(dest_dir)
        .map_err(|e| AppError::ArchiveExtract(e.to_string()))
}

/// Core logic for single-entry extraction: iterate the tar.gz stream and extract the
/// matching file or directory entry to `dest`.
///
/// - File entry: exact normalised-path match → written directly to `dest`.
/// - Directory entry: prefix match → contents recreated under `dest/`.
/// - Symlink as the specified entry → error.
/// - Symlink as a child during directory extraction → warning + skip.
/// - No match → error listing top-level archive entries.
fn extract_archive_entry<R: Read>(reader: R, entry: &str, dest: &Path) -> Result<(), AppError> {
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);

    let norm_entry = normalize_entry_path(entry).to_string();
    let dir_prefix = format!("{}/", norm_entry);

    let mut matched = false;
    let mut top_level: Vec<String> = Vec::new();

    for entry_result in archive
        .entries()
        .map_err(|e| AppError::ArchiveRead(e.to_string()))?
    {
        let mut tar_entry = entry_result.map_err(|e| AppError::ArchiveRead(e.to_string()))?;

        let path_owned = tar_entry
            .path()
            .map_err(|e| AppError::ArchiveRead(e.to_string()))?
            .to_string_lossy()
            .into_owned();
        let norm_path = normalize_entry_path(&path_owned).to_string();

        // Track unique top-level components for the not-found error message.
        if let Some(first) = norm_path.split('/').next()
            && !first.is_empty()
            && !top_level.contains(&first.to_string())
        {
            top_level.push(first.to_string());
        }

        // Skip directory meta-entries; actual files carry the content.
        if tar_entry.header().entry_type().is_dir() {
            continue;
        }

        // ── Exact file-entry match ────────────────────────────────────────────
        if norm_path == norm_entry {
            if tar_entry.header().entry_type().is_symlink() {
                return Err(AppError::SymlinkEntry(norm_entry));
            }
            if let Some(parent) = dest.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }
            tar_entry
                .unpack(dest)
                .map_err(|e| AppError::ArchiveExtract(e.to_string()))?;
            matched = true;
            break; // unique match — stop streaming
        }

        // ── Directory-entry prefix match ─────────────────────────────────────
        if norm_path.starts_with(&dir_prefix) {
            if tar_entry.header().entry_type().is_symlink() {
                eprintln!("Warning: skipping symlink entry '{}' in archive", norm_path);
                continue;
            }
            let relative = &norm_path[dir_prefix.len()..];
            let dest_path = dest.join(relative);
            if let Some(parent) = dest_path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }
            tar_entry
                .unpack(&dest_path)
                .map_err(|e| AppError::ArchiveExtract(e.to_string()))?;
            matched = true;
        }
    }

    if !matched {
        top_level.sort();
        return Err(AppError::EntryNotFound(norm_entry, top_level.join("\n  ")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn test_archive_path_has_preceeding_and_trailing_slash() {
        assert_eq!(normalize_entry_path("./foo/bar/"), "foo/bar");
    }

    // ── Destination::resolve ─────────────────────────────────────────────────

    #[test]
    fn test_destination_resolve_no_args_uses_current_dir() {
        let dest = Destination::resolve(None, None).unwrap();
        assert_matches!(dest, Destination::Into(d) if d == std::path::Path::new("."));
    }

    #[test]
    fn test_destination_resolve_into_existing_dir_appends_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = Destination::resolve(Some(tmp.path()), None).unwrap();
        assert_matches!(dest, Destination::Into(d) if d == tmp.path());
    }

    #[test]
    fn test_destination_resolve_into_non_existing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let non_existing = tmp.path().join("subdir");
        let dest = Destination::resolve(Some(&non_existing), None).unwrap();
        assert_matches!(dest, Destination::Into(d) if d == non_existing);
    }

    #[test]
    fn test_destination_resolve_exact_non_existing_path_used_as_is() {
        let dest =
            Destination::resolve(None, Some(std::path::Path::new("/tmp/renamed.bin"))).unwrap();
        assert_matches!(dest, Destination::Exact(p) if p == std::path::Path::new("/tmp/renamed.bin"));
    }

    #[test]
    fn test_destination_resolve_exact_existing_file_used_as_is() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("existing.bin");
        std::fs::File::create(&file_path).unwrap();
        let dest = Destination::resolve(None, Some(&file_path)).unwrap();
        assert_matches!(dest, Destination::Exact(p) if p == file_path);
    }

    #[test]
    fn test_destination_resolve_exact_existing_directory_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = Destination::resolve(None, Some(tmp.path())).unwrap_err();
        assert_matches!(err, AppError::OutputIsDir(_));
    }

    /// Build an in-memory .tar.gz with the given (archive-path, content) pairs.
    /// Pass `None` as content to create a symlink entry (target = "symlink-target").
    fn make_tar_gz_with_entries(entries: &[(&str, Option<&str>)]) -> Vec<u8> {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let gz = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = tar::Builder::new(gz);

        for (path, content) in entries {
            match content {
                Some(data) => {
                    let bytes = data.as_bytes();
                    let mut header = tar::Header::new_gnu();
                    header.set_size(bytes.len() as u64);
                    header.set_mode(0o644);
                    header.set_cksum();
                    builder
                        .append_data(&mut header, path, std::io::Cursor::new(bytes))
                        .unwrap();
                }
                None => {
                    let mut header = tar::Header::new_gnu();
                    header.set_entry_type(tar::EntryType::Symlink);
                    header.set_mode(0o777);
                    header.set_size(0);
                    builder
                        .append_link(&mut header, path, "symlink-target")
                        .unwrap();
                }
            }
        }

        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap()
    }

    #[test]
    fn test_is_extractable() {
        assert!(is_extractable("tool-v1.0.0-linux-amd64.tar.gz"));
        assert!(is_extractable("tool-v1.0.0-linux-amd64.tgz"));
        assert!(!is_extractable("tool-v1.0.0-linux-amd64.zip"));
        assert!(!is_extractable("tool-v1.0.0-linux-amd64.tar.bz2"));
        assert!(!is_extractable("tool-v1.0.0-linux-amd64"));
    }

    #[test]
    fn test_unpack_tar_gz_extracts_files() {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let src_dir = tempfile::tempdir().unwrap();
        let src_file = src_dir.path().join("hello.txt");
        std::fs::write(&src_file, "hello from tarball").unwrap();

        let gz = GzEncoder::new(Vec::new(), Compression::default());
        let mut archive = tar::Builder::new(gz);
        archive
            .append_path_with_name(&src_file, "hello.txt")
            .unwrap();
        let gz = archive.into_inner().unwrap();
        let gz_data = gz.finish().unwrap();

        let dest_dir = tempfile::tempdir().unwrap();
        unpack_tar_gz(gz_data.as_slice(), dest_dir.path()).unwrap();

        let extracted = dest_dir.path().join("hello.txt");
        assert!(extracted.exists(), "expected hello.txt to be extracted");
        assert_eq!(
            std::fs::read_to_string(&extracted).unwrap(),
            "hello from tarball"
        );
    }

    // ── whole-archive extraction ─────────────────────────────────────────────

    #[test]
    fn test_extract_whole_archive_into_dir() {
        let data = make_tar_gz_with_entries(&[
            ("bin/tool", Some("binary")),
            ("README.md", Some("readme")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest_dir = tmp.path().join("out");
        let landing =
            extract_archive(data.as_slice(), None, Destination::Into(dest_dir.clone())).unwrap();
        assert_eq!(landing, dest_dir);
        assert!(dest_dir.join("bin/tool").exists());
        assert!(dest_dir.join("README.md").exists());
    }

    // ── file entry → Into (default / --dir) ──────────────────────────

    #[test]
    fn test_extract_entry_file_into_dir() {
        let data = make_tar_gz_with_entries(&[
            ("bin/tool", Some("binary content")),
            ("README.md", Some("readme")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let landing = extract_archive(
            data.as_slice(),
            Some("bin/tool"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert_eq!(landing, tmp.path().join("tool"));
        assert_eq!(std::fs::read_to_string(&landing).unwrap(), "binary content");
    }

    // ── file entry → Exact (--output) ────────────────────────────────

    #[test]
    fn test_extract_entry_file_to_exact() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("renamed content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("mytool");
        let landing = extract_archive(
            data.as_slice(),
            Some("bin/tool"),
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(landing, dest);
        assert_eq!(
            std::fs::read_to_string(&landing).unwrap(),
            "renamed content"
        );
    }

    // ── directory entry → Into (default / --dir) ───────────────────────────────────────

    #[test]
    fn test_extract_entry_dir_into_dir() {
        let data = make_tar_gz_with_entries(&[
            ("share/config/a.conf", Some("aaa")),
            ("share/config/b.conf", Some("bbb")),
            ("other/file.txt", Some("other")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let landing = extract_archive(
            data.as_slice(),
            Some("share/config"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert_eq!(landing, tmp.path().join("config"));
        assert_eq!(
            std::fs::read_to_string(landing.join("a.conf")).unwrap(),
            "aaa"
        );
        assert_eq!(
            std::fs::read_to_string(landing.join("b.conf")).unwrap(),
            "bbb"
        );
    }

    // ── directory entry → Exact (--output, rename root) ──────────────────────

    #[test]
    fn test_extract_entry_dir_to_exact() {
        let data = make_tar_gz_with_entries(&[
            ("share/config/a.conf", Some("aaa")),
            ("share/config/sub/b.conf", Some("bbb")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("myconfig");
        let landing = extract_archive(
            data.as_slice(),
            Some("share/config"),
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(landing, dest);
        assert_eq!(
            std::fs::read_to_string(landing.join("a.conf")).unwrap(),
            "aaa"
        );
        assert_eq!(
            std::fs::read_to_string(landing.join("sub/b.conf")).unwrap(),
            "bbb"
        );
    }

    // ── directory entry → merges into existing dest ──────────────────────────

    #[test]
    fn test_extract_entry_dir_merges_into_existing_dest() {
        let data = make_tar_gz_with_entries(&[
            ("mydir/foo/bar", Some("from archive")),
            ("mydir/foo/baz", Some("also from archive")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("out");
        std::fs::create_dir_all(dest.join("foo")).unwrap();
        std::fs::write(dest.join("foo/bar"), "original").unwrap();
        std::fs::write(dest.join("foo/quux"), "also original").unwrap();

        extract_archive(
            data.as_slice(),
            Some("mydir"),
            Destination::Exact(dest.clone()),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.join("foo/bar")).unwrap(),
            "from archive",
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("foo/baz")).unwrap(),
            "also from archive",
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("foo/quux")).unwrap(),
            "also original",
        );
    }

    // ── entry not found → error lists top-level entries ─────────────────────

    #[test]
    fn test_extract_entry_not_found_lists_top_level() {
        let data =
            make_tar_gz_with_entries(&[("bin/tool", Some("x")), ("share/man/tool.1", Some("y"))]);
        let tmp = tempfile::tempdir().unwrap();
        let err = extract_archive(
            data.as_slice(),
            Some("no/such/path"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap_err();
        assert_matches!(err, AppError::EntryNotFound(entry, top_level)
            if entry == "no/such/path" && top_level.contains("bin") && top_level.contains("share")
        );
    }

    // ── directly specified symlink entry → error ─────────────────────────────

    #[test]
    fn test_extract_entry_symlink_direct_returns_error() {
        let data = make_tar_gz_with_entries(&[("bin/tool", None)]);
        let tmp = tempfile::tempdir().unwrap();
        let err = extract_archive(
            data.as_slice(),
            Some("bin/tool"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap_err();
        assert_matches!(err, AppError::SymlinkEntry(entry) if entry == "bin/tool");
    }

    // ── child symlinks during directory extraction → skip + warn ────────────

    #[test]
    fn test_extract_entry_dir_child_symlinks_skipped() {
        let data =
            make_tar_gz_with_entries(&[("pkg/real.txt", Some("real")), ("pkg/link.txt", None)]);
        let tmp = tempfile::tempdir().unwrap();
        let landing = extract_archive(
            data.as_slice(),
            Some("pkg"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert!(landing.join("real.txt").exists());
        assert!(!landing.join("link.txt").exists());
    }

    // ── parent directories created automatically ─────────────────────────────

    #[test]
    fn test_extract_entry_creates_parent_dirs() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("new/nested/dir/tool");
        let landing = extract_archive(
            data.as_slice(),
            Some("bin/tool"),
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(std::fs::read_to_string(&landing).unwrap(), "content");
    }

    // ── save_to_file ─────────────────────────────────────────────────────────

    #[test]
    fn test_save_to_file_into_writes_file_with_asset_name() {
        let tmp = tempfile::tempdir().unwrap();
        let content = b"hello bytes";
        let landing = save_to_file(
            content.as_slice(),
            "asset.bin",
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert_eq!(landing, tmp.path().join("asset.bin"));
        assert_eq!(std::fs::read(&landing).unwrap(), content);
    }

    #[test]
    fn test_save_to_file_exact_writes_file_to_given_path() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("renamed.bin");
        let content = b"exact content";
        let landing = save_to_file(
            content.as_slice(),
            "ignored_name",
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(landing, dest);
        assert_eq!(std::fs::read(&landing).unwrap(), content);
    }

    #[test]
    fn test_save_to_file_exact_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("a/b/c/file.bin");
        let content = b"nested";
        let landing = save_to_file(
            content.as_slice(),
            "ignored",
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(landing, dest);
        assert_eq!(std::fs::read(&landing).unwrap(), content);
    }

    #[test]
    fn test_save_to_file_into_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("new/nested/dir");
        let content = b"data";
        let landing = save_to_file(
            content.as_slice(),
            "file.bin",
            Destination::Into(dir.clone()),
        )
        .unwrap();
        assert_eq!(landing, dir.join("file.bin"));
        assert_eq!(std::fs::read(&landing).unwrap(), content);
    }

    #[test]
    fn test_save_to_file_overwrites_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("file.bin");
        std::fs::write(&dest, "old content").unwrap();
        let content = b"new content";
        save_to_file(
            content.as_slice(),
            "ignored",
            Destination::Exact(dest.clone()),
        )
        .unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), content);
    }

    #[test]
    fn test_save_to_file_returns_error_when_parent_is_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        // Create a file where a directory is expected as parent
        let blocking_file = tmp.path().join("notadir");
        std::fs::write(&blocking_file, "i am a file").unwrap();
        let dest = blocking_file.join("child.bin");
        let err =
            save_to_file(b"data".as_slice(), "ignored", Destination::Exact(dest)).unwrap_err();
        assert_matches!(err, AppError::CreateDir { .. });
    }
}
