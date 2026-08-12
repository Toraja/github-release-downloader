use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn resolve_output_path(
    asset_name: &str,
    dir: Option<&Path>,
    output: Option<&Path>,
) -> Result<PathBuf, AppError> {
    if let Some(output) = output {
        if output.is_dir() {
            return Err(AppError::OutputIsDir(output.display().to_string()));
        }
        return Ok(output.to_path_buf());
    }

    let base = dir.unwrap_or_else(|| Path::new("."));
    Ok(base.join(asset_name))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_resolve_no_flags_uses_current_dir() {
        let path = resolve_output_path("asset.tar.gz", None, None).unwrap();
        assert_eq!(path, std::path::Path::new(".").join("asset.tar.gz"));
    }

    #[test]
    fn test_resolve_dir_existing_appends_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let path = resolve_output_path("asset.tar.gz", Some(tmp.path()), None).unwrap();
        assert_eq!(path, tmp.path().join("asset.tar.gz"));
    }

    #[test]
    fn test_resolve_dir_non_existing_appends_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let non_existing = tmp.path().join("subdir");
        let path = resolve_output_path("asset.tar.gz", Some(&non_existing), None).unwrap();
        assert_eq!(path, non_existing.join("asset.tar.gz"));
    }

    #[test]
    fn test_resolve_output_non_existing_path_used_as_is() {
        let path = resolve_output_path(
            "asset.tar.gz",
            None,
            Some(std::path::Path::new("/tmp/renamed.bin")),
        )
        .unwrap();
        assert_eq!(path, std::path::Path::new("/tmp/renamed.bin"));
    }

    #[test]
    fn test_resolve_output_existing_file_used_as_is() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("existing.bin");
        File::create(&file_path).unwrap();
        let path = resolve_output_path("asset.tar.gz", None, Some(&file_path)).unwrap();
        assert_eq!(path, file_path);
    }

    #[test]
    fn test_resolve_output_existing_directory_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = resolve_output_path("asset.tar.gz", None, Some(tmp.path())).unwrap_err();
        assert!(err.to_string().contains("existing directory"));
    }
}
