use std::path::PathBuf;

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
    ) -> Result<Self, AppError> {
        match exact {
            Some(p) => {
                if p.is_dir() {
                    return Err(AppError::OutputIsDir(p.display().to_string()));
                }
                Ok(Destination::Exact(p.to_path_buf()))
            }
            None => {
                let p = into.unwrap_or(std::path::Path::new("."));
                if p.is_file() {
                    return Err(AppError::DirIsFile(p.display().to_string()));
                }
                Ok(Destination::Into(p.to_path_buf()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

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

    #[test]
    fn test_destination_resolve_into_existing_file_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("existing.bin");
        std::fs::File::create(&file_path).unwrap();
        let err = Destination::resolve(Some(&file_path), None).unwrap_err();
        assert_matches!(err, AppError::DirIsFile(_));
    }
}
