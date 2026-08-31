use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use zip::ZipArchive;

use crate::destination::Destination;
use crate::error::AppError;

/// Archive formats supported for extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    TarGz,
    Zip,
}

/// Detect the archive format from an asset filename, if supported.
pub fn detect_format(asset_name: &str) -> Option<ArchiveFormat> {
    if asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz") {
        Some(ArchiveFormat::TarGz)
    } else if asset_name.ends_with(".zip") {
        Some(ArchiveFormat::Zip)
    } else {
        None
    }
}

/// Strip a leading `./` and a trailing `/` from an archive entry path.
fn normalize_entry_path(s: &str) -> &str {
    let s = s.strip_prefix("./").unwrap_or(s);
    s.trim_end_matches('/')
}

/// One implementation per supported archive format (see openspec design).
trait Extractor {
    /// Unpack the entire archive into `dest_dir`.
    fn unpack_all(&mut self, dest_dir: &Path) -> Result<(), AppError>;

    /// Extract the file or directory entry `entry` to `dest`,
    /// applying the shared matching/symlink/not-found rules.
    fn extract_entry(&mut self, entry: &str, dest: &Path) -> Result<(), AppError>;
}

struct TarGzExtractor<R: Read> {
    reader: R,
}

impl<R: Read> Extractor for TarGzExtractor<R> {
    fn unpack_all(&mut self, dest_dir: &Path) -> Result<(), AppError> {
        let gz = GzDecoder::new(&mut self.reader);
        let mut archive = tar::Archive::new(gz);
        archive
            .unpack(dest_dir)
            .map_err(|e| AppError::ArchiveExtract(e.to_string()))
    }

    fn extract_entry(&mut self, entry: &str, dest: &Path) -> Result<(), AppError> {
        let gz = GzDecoder::new(&mut self.reader);
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

            collect_top_level(&norm_path, &mut top_level);

            // Skip directory meta-entries; actual files carry the content.
            if tar_entry.header().entry_type().is_dir() {
                continue;
            }

            if norm_path == norm_entry {
                if tar_entry.header().entry_type().is_symlink() {
                    return Err(AppError::SymlinkEntry(norm_entry));
                }
                write_file_entry(&mut tar_entry, dest)?;
                matched = true;
                break; // unique match — stop streaming
            }

            if norm_path.starts_with(&dir_prefix) {
                if tar_entry.header().entry_type().is_symlink() {
                    eprintln!("Warning: skipping symlink entry '{}' in archive", norm_path);
                    continue;
                }
                let relative = &norm_path[dir_prefix.len()..];
                let dest_path = dest.join(relative);
                write_file_entry(&mut tar_entry, &dest_path)?;
                matched = true;
            }
        }

        if !matched {
            top_level.sort();
            return Err(AppError::EntryNotFound(norm_entry, top_level.join("\n  ")));
        }

        Ok(())
    }
}

struct ZipExtractor {
    data: Vec<u8>,
}

impl ZipExtractor {
    fn new(reader: impl Read) -> Result<Self, AppError> {
        let mut data = Vec::new();
        let mut reader = reader;
        reader
            .read_to_end(&mut data)
            .map_err(|e| AppError::ArchiveRead(e.to_string()))?;
        Ok(Self { data })
    }

    /// Build a fresh `ZipArchive` view over the owned buffer on each call,
    /// avoiding a self-referential borrow.
    fn archive(&self) -> Result<ZipArchive<Cursor<&[u8]>>, AppError> {
        ZipArchive::new(Cursor::new(self.data.as_slice()))
            .map_err(|e| AppError::ArchiveRead(e.to_string()))
    }
}

impl Extractor for ZipExtractor {
    fn unpack_all(&mut self, dest_dir: &Path) -> Result<(), AppError> {
        let mut archive = self.archive()?;
        archive
            .extract(dest_dir)
            .map_err(|e| AppError::ArchiveExtract(e.to_string()))
    }

    fn extract_entry(&mut self, entry: &str, dest: &Path) -> Result<(), AppError> {
        let mut archive = self.archive()?;

        let norm_entry = normalize_entry_path(entry).to_string();
        let dir_prefix = format!("{}/", norm_entry);

        let mut matched = false;
        let mut top_level: Vec<String> = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::ArchiveRead(e.to_string()))?;

            // enclosed_name() sanitises path traversal; also normalise `\` separators
            // for archives produced on Windows.
            let raw_name = file
                .enclosed_name()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let path_owned = raw_name.replace('\\', "/");
            let norm_path = normalize_entry_path(&path_owned).to_string();

            if norm_path.is_empty() {
                continue; // unsafe or empty entry name
            }

            collect_top_level(&norm_path, &mut top_level);

            // Skip directory entries; actual files carry the content.
            if file.is_dir() {
                continue;
            }

            // Symlink detection via unix mode high bits (S_IFLNK). Entries without
            // a unix mode (e.g. archives produced on Windows) are treated as files.
            let is_symlink = file.unix_mode().is_some_and(|m| (m & 0o170000) == 0o120000);

            if norm_path == norm_entry {
                if is_symlink {
                    return Err(AppError::SymlinkEntry(norm_entry));
                }
                write_zip_entry(&mut file, dest)?;
                matched = true;
                break; // unique match
            }

            if norm_path.starts_with(&dir_prefix) {
                if is_symlink {
                    eprintln!("Warning: skipping symlink entry '{}' in archive", norm_path);
                    continue;
                }
                let relative = &norm_path[dir_prefix.len()..];
                let dest_path = dest.join(relative);
                write_zip_entry(&mut file, &dest_path)?;
                matched = true;
            }
        }

        if !matched {
            top_level.sort();
            return Err(AppError::EntryNotFound(norm_entry, top_level.join("\n  ")));
        }

        Ok(())
    }
}

/// Track unique top-level path components for the not-found error message.
fn collect_top_level(norm_path: &str, top_level: &mut Vec<String>) {
    if let Some(first) = norm_path.split('/').next()
        && !first.is_empty()
        && !top_level.contains(&first.to_string())
    {
        top_level.push(first.to_string());
    }
}

/// Create missing parent directories for `dest` and unpack a tar entry into it.
fn write_file_entry(entry: &mut tar::Entry<'_, impl Read>, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
            path: parent.display().to_string(),
            source: e,
        })?;
    }
    entry
        .unpack(dest)
        .map_err(|e| AppError::ArchiveExtract(e.to_string()))?;
    Ok(())
}

/// Create missing parent directories for `dest` and unpack a zip entry into it.
fn write_zip_entry<R: Read>(
    file: &mut zip::read::ZipFile<'_, R>,
    dest: &Path,
) -> Result<(), AppError> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| AppError::CreateDir {
            path: parent.display().to_string(),
            source: e,
        })?;
    }
    let mut out = fs::File::create(dest).map_err(|e| AppError::CreateFile {
        path: dest.display().to_string(),
        source: e,
    })?;
    std::io::copy(file, &mut out).map_err(|e| AppError::ArchiveExtract(e.to_string()))?;
    Ok(())
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
    format: ArchiveFormat,
    entry: Option<&str>,
    dest: Destination,
) -> Result<PathBuf, AppError> {
    let mut extractor: Box<dyn Extractor> = match format {
        ArchiveFormat::TarGz => Box::new(TarGzExtractor { reader }),
        ArchiveFormat::Zip => Box::new(ZipExtractor::new(reader)?),
    };

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
            extractor.unpack_all(&dir)?;
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
            extractor.extract_entry(entry_path, &landing)?;
            Ok(landing)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::destination::Destination;

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

    /// Build an in-memory .zip with the given (archive-path, content) pairs.
    /// Pass `None` as content to create a symlink entry (target = "symlink-target",
    /// marked via unix mode S_IFLNK).
    fn make_zip_with_entries(entries: &[(&str, Option<&str>)]) -> Vec<u8> {
        use zip::write::SimpleFileOptions;

        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (path, content) in entries {
            match content {
                Some(data) => {
                    writer
                        .start_file(path, SimpleFileOptions::default())
                        .unwrap();
                    std::io::Write::write_all(&mut writer, data.as_bytes()).unwrap();
                }
                None => {
                    let options = SimpleFileOptions::default().unix_permissions(0o777);
                    writer.add_symlink(path, "symlink-target", options).unwrap();
                }
            }
        }
        writer.finish().unwrap().into_inner()
    }

    /// Build an in-memory archive in the given format (None content = symlink for
    /// tar.gz; zip symlink support is tested separately below).
    fn make_archive(format: ArchiveFormat, entries: &[(&str, Option<&str>)]) -> Vec<u8> {
        match format {
            ArchiveFormat::TarGz => make_tar_gz_with_entries(entries),
            ArchiveFormat::Zip => make_zip_with_entries(entries),
        }
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            detect_format("tool-v1.0.0-linux-amd64.tar.gz"),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            detect_format("tool-v1.0.0-linux-amd64.tgz"),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            detect_format("tool-v1.0.0-linux-amd64.zip"),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(detect_format("tool-v1.0.0-linux-amd64.tar.bz2"), None);
        assert_eq!(detect_format("tool-v1.0.0-linux-amd64"), None);
    }

    // ── whole-archive extraction (both formats) ────────────────────────────

    #[test]
    fn test_extract_whole_archive_into_dir() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[("bin/tool", Some("binary")), ("README.md", Some("readme"))],
            );
            let tmp = tempfile::tempdir().unwrap();
            let dest_dir = tmp.path().join("out");
            let landing = extract_archive(
                data.as_slice(),
                format,
                None,
                Destination::Into(dest_dir.clone()),
            )
            .unwrap();
            assert_eq!(landing, dest_dir);
            assert!(dest_dir.join("bin/tool").exists());
            assert!(dest_dir.join("README.md").exists());
        }
    }

    // ── file entry → Into (default / --dir) ─────────────────────────────────

    #[test]
    fn test_extract_entry_file_into_dir() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[
                    ("bin/tool", Some("binary content")),
                    ("README.md", Some("readme")),
                ],
            );
            let tmp = tempfile::tempdir().unwrap();
            let landing = extract_archive(
                data.as_slice(),
                format,
                Some("bin/tool"),
                Destination::Into(tmp.path().to_path_buf()),
            )
            .unwrap();
            assert_eq!(landing, tmp.path().join("tool"));
            assert_eq!(std::fs::read_to_string(&landing).unwrap(), "binary content");
        }
    }

    // ── file entry → Exact (--output) ───────────────────────────────────────

    #[test]
    fn test_extract_entry_file_to_exact() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(format, &[("bin/tool", Some("renamed content"))]);
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("mytool");
            let landing = extract_archive(
                data.as_slice(),
                format,
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
    }

    // ── directory entry → Into (default / --dir) ────────────────────────────

    #[test]
    fn test_extract_entry_dir_into_dir() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[
                    ("share/config/a.conf", Some("aaa")),
                    ("share/config/b.conf", Some("bbb")),
                    ("other/file.txt", Some("other")),
                ],
            );
            let tmp = tempfile::tempdir().unwrap();
            let landing = extract_archive(
                data.as_slice(),
                format,
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
    }

    // ── directory entry → Exact (--output, rename root) ─────────────────────

    #[test]
    fn test_extract_entry_dir_to_exact() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[
                    ("share/config/a.conf", Some("aaa")),
                    ("share/config/sub/b.conf", Some("bbb")),
                ],
            );
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("myconfig");
            let landing = extract_archive(
                data.as_slice(),
                format,
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
    }

    // ── directory entry → merges into existing dest ─────────────────────────

    #[test]
    fn test_extract_entry_dir_merges_into_existing_dest() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[
                    ("mydir/foo/bar", Some("from archive")),
                    ("mydir/foo/baz", Some("also from archive")),
                ],
            );
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("out");
            std::fs::create_dir_all(dest.join("foo")).unwrap();
            std::fs::write(dest.join("foo/bar"), "original").unwrap();
            std::fs::write(dest.join("foo/quux"), "also original").unwrap();

            extract_archive(
                data.as_slice(),
                format,
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
    }

    // ── entry not found → error lists top-level entries ─────────────────────

    #[test]
    fn test_extract_entry_not_found_lists_top_level() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[("bin/tool", Some("x")), ("share/man/tool.1", Some("y"))],
            );
            let tmp = tempfile::tempdir().unwrap();
            let err = extract_archive(
                data.as_slice(),
                format,
                Some("no/such/path"),
                Destination::Into(tmp.path().to_path_buf()),
            )
            .unwrap_err();
            assert_matches!(err, AppError::EntryNotFound(entry, top_level)
                if entry == "no/such/path" && top_level.contains("bin") && top_level.contains("share")
            );
        }
    }

    // ── directly specified symlink entry → error ────────────────────────────

    #[test]
    fn test_extract_entry_symlink_direct_returns_error() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(format, &[("bin/tool", None)]);
            let tmp = tempfile::tempdir().unwrap();
            let err = extract_archive(
                data.as_slice(),
                format,
                Some("bin/tool"),
                Destination::Into(tmp.path().to_path_buf()),
            )
            .unwrap_err();
            assert_matches!(err, AppError::SymlinkEntry(entry) if entry == "bin/tool");
        }
    }

    // ── child symlinks during directory extraction → skip + warn ────────────

    #[test]
    fn test_extract_entry_dir_child_symlinks_skipped() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(
                format,
                &[("pkg/real.txt", Some("real")), ("pkg/link.txt", None)],
            );
            let tmp = tempfile::tempdir().unwrap();
            let landing = extract_archive(
                data.as_slice(),
                format,
                Some("pkg"),
                Destination::Into(tmp.path().to_path_buf()),
            )
            .unwrap();
            assert!(landing.join("real.txt").exists());
            assert!(!landing.join("link.txt").exists());
        }
    }

    // ── parent directories created automatically ────────────────────────────

    #[test]
    fn test_extract_entry_creates_parent_dirs() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(format, &[("bin/tool", Some("content"))]);
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("new/nested/dir/tool");
            let landing = extract_archive(
                data.as_slice(),
                format,
                Some("bin/tool"),
                Destination::Exact(dest.clone()),
            )
            .unwrap();
            assert_eq!(std::fs::read_to_string(&landing).unwrap(), "content");
        }
    }

    // ── no archive file left on disk after entry extraction ─────────────────

    #[test]
    fn test_extract_entry_leaves_no_archive_file() {
        for format in [ArchiveFormat::TarGz, ArchiveFormat::Zip] {
            let data = make_archive(format, &[("bin/tool", Some("content"))]);
            let tmp = tempfile::tempdir().unwrap();
            extract_archive(
                data.as_slice(),
                format,
                Some("bin/tool"),
                Destination::Into(tmp.path().to_path_buf()),
            )
            .unwrap();

            let leftovers: Vec<_> = std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.ends_with(".tar.gz") || n.ends_with(".tgz") || n.ends_with(".zip")
                })
                .collect();
            assert!(leftovers.is_empty(), "archive file left on disk");
        }
    }

    // ── zip handles backslash separators in entry names ────────────────────

    #[test]
    fn test_zip_entry_with_backslash_separators() {
        // Hand-build a zip whose entry name uses `\` separators (Windows-produced).
        use zip::write::SimpleFileOptions;

        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file("bin\\tool", SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut writer, b"backslash content").unwrap();
        let data = writer.finish().unwrap().into_inner();

        let tmp = tempfile::tempdir().unwrap();
        let landing = extract_archive(
            data.as_slice(),
            ArchiveFormat::Zip,
            Some("bin/tool"),
            Destination::Into(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(&landing).unwrap(),
            "backslash content"
        );
    }
}
