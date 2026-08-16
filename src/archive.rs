use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use flate2::read::GzDecoder;

use crate::error::AppError;

pub fn is_extractable(asset_name: &str) -> bool {
    asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz")
}

/// Strip a leading `./` and a trailing `/` from an archive entry path.
pub fn normalize_entry_path(s: &str) -> &str {
    let s = s.strip_prefix("./").unwrap_or(s);
    s.trim_end_matches('/')
}

pub fn save_to_file(reader: impl Read, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    let mut file = File::create(dest).map_err(|e| AppError::CreateFile {
        path: dest.display().to_string(),
        source: e,
    })?;

    let mut reader = reader;
    io::copy(&mut reader, &mut file).map_err(|e| AppError::WriteFile {
        path: dest.display().to_string(),
        source: e,
    })?;

    Ok(())
}

pub fn extract_archive(reader: impl Read, dest_dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dest_dir).map_err(|e| AppError::CreateDir {
        path: dest_dir.display().to_string(),
        source: e,
    })?;

    unpack_tar_gz(reader, dest_dir)
}

pub fn unpack_tar_gz<R: Read>(reader: R, dest_dir: &Path) -> Result<(), AppError> {
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(dest_dir)
        .map_err(|e| AppError::ArchiveExtract(e.to_string()))
}

/// Core logic for `--extract-entry`: iterate the tar.gz stream and extract the
/// matching file or directory entry to `dest`.
///
/// - File entry: exact normalised-path match → written directly to `dest`.
/// - Directory entry: prefix match → contents recreated under `dest/`.
/// - Symlink as the specified entry → error.
/// - Symlink as a child during directory extraction → warning + skip.
/// - No match → error listing top-level archive entries.
pub fn extract_archive_entry<R: Read>(reader: R, entry: &str, dest: &Path) -> Result<(), AppError> {
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
    use super::*;

    #[test]
    fn test_archive_path_has_preceeding_and_trailing_slash() {
        assert_eq!(normalize_entry_path("./foo/bar/"), "foo/bar");
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
    fn test_extract_entry_unsupported_format() {
        assert!(!is_extractable("tool-v1.0.zip"));
        assert!(!is_extractable("tool-v1.0.tar.bz2"));
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

    // ── file entry → default destination ────────────────────────────────────

    #[test]
    fn test_extract_entry_file_default_dest() {
        let data = make_tar_gz_with_entries(&[
            ("bin/tool", Some("binary content")),
            ("README.md", Some("readme")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("tool");
        extract_archive_entry(data.as_slice(), "bin/tool", &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "binary content");
    }

    // ── file entry → --dir ───────────────────────────────────────────────────

    #[test]
    fn test_extract_entry_file_with_dir() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("binary"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("tool");
        extract_archive_entry(data.as_slice(), "bin/tool", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "binary");
    }

    // ── file entry → --output (rename) ──────────────────────────────────────

    #[test]
    fn test_extract_entry_file_with_output_rename() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("renamed content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("mytool");
        extract_archive_entry(data.as_slice(), "bin/tool", &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "renamed content");
    }

    // ── directory entry → default destination ───────────────────────────────

    #[test]
    fn test_extract_entry_dir_default_dest() {
        let data = make_tar_gz_with_entries(&[
            ("share/config/a.conf", Some("aaa")),
            ("share/config/b.conf", Some("bbb")),
            ("other/file.txt", Some("other")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("config");
        extract_archive_entry(data.as_slice(), "share/config", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("a.conf")).unwrap(), "aaa");
        assert_eq!(std::fs::read_to_string(dest.join("b.conf")).unwrap(), "bbb");
        assert!(!tmp.path().join("config").join("../other").exists());
    }

    // ── directory entry → --dir ──────────────────────────────────────────────

    #[test]
    fn test_extract_entry_dir_with_dir_flag() {
        let data = make_tar_gz_with_entries(&[("pkg/lib/x.so", Some("lib"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("lib");
        extract_archive_entry(data.as_slice(), "pkg/lib", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("x.so")).unwrap(), "lib");
    }

    // ── directory entry → --output (rename root) ────────────────────────────

    #[test]
    fn test_extract_entry_dir_with_output_rename() {
        let data = make_tar_gz_with_entries(&[
            ("share/config/a.conf", Some("aaa")),
            ("share/config/sub/b.conf", Some("bbb")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("myconfig");
        extract_archive_entry(data.as_slice(), "share/config", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("a.conf")).unwrap(), "aaa");
        assert_eq!(
            std::fs::read_to_string(dest.join("sub/b.conf")).unwrap(),
            "bbb"
        );
    }

    // ── directory entry → merges into existing dest, overlapping file overwritten

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

        extract_archive_entry(data.as_slice(), "mydir", &dest).unwrap();

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
        let dest = tmp.path().join("out");
        let err = extract_archive_entry(data.as_slice(), "no/such/path", &dest).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not found"), "expected 'not found' in: {msg}");
        assert!(msg.contains("bin"), "expected top-level 'bin' in: {msg}");
        assert!(
            msg.contains("share"),
            "expected top-level 'share' in: {msg}"
        );
    }

    // ── directly specified symlink entry → error ─────────────────────────────

    #[test]
    fn test_extract_entry_symlink_direct_returns_error() {
        let data = make_tar_gz_with_entries(&[("bin/tool", None)]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("tool");
        let err = extract_archive_entry(data.as_slice(), "bin/tool", &dest).unwrap_err();
        assert!(
            err.to_string().contains("symlink"),
            "expected 'symlink' in error: {err}"
        );
    }

    // ── child symlinks during directory extraction → skip + warn ────────────

    #[test]
    fn test_extract_entry_dir_child_symlinks_skipped() {
        let data =
            make_tar_gz_with_entries(&[("pkg/real.txt", Some("real")), ("pkg/link.txt", None)]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("pkg");
        extract_archive_entry(data.as_slice(), "pkg", &dest).unwrap();
        assert!(dest.join("real.txt").exists());
        assert!(!dest.join("link.txt").exists());
    }

    // ── parent directories created automatically ─────────────────────────────

    #[test]
    fn test_extract_entry_creates_parent_dirs() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("new/nested/dir/tool");
        extract_archive_entry(data.as_slice(), "bin/tool", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "content");
    }
}
