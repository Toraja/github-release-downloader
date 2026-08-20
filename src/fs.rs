use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

use crate::destination::Destination;
use crate::error::AppError;

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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    use crate::destination::Destination;

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
