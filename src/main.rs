use std::path::Path;

use clap::Parser;
use regex::Regex;
use url::Url;

mod archive;
mod error;
mod github;
mod output;

use archive::{
    extract_archive, extract_archive_entry, is_extractable, normalize_entry_path, save_to_file,
};
use error::AppError;
use github::{fetch_asset, fetch_release, select_asset, to_api_url};
use output::resolve_output_path;

/// Download a release asset from a GitHub repository.
///
/// Fetches the latest release from the given GitHub repository and downloads
/// the single asset whose name matches PATTERN (a regular expression).
///
/// Authentication: set the GITHUB_TOKEN environment variable to use an
/// authenticated request and avoid the 60 req/hr unauthenticated rate limit.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[allow(rustdoc::bare_urls)]
    /// GitHub repository URL (e.g., https://github.com/owner/repo)
    url: Url,

    /// Regex pattern to match against asset names (must match exactly one asset)
    pattern: Regex,

    /// Directory in which to save the downloaded asset (original filename is preserved).
    /// Parent directories are created automatically if they do not exist.
    /// Mutually exclusive with --output.
    #[arg(short = 'D', long, conflicts_with = "output")]
    dir: Option<std::path::PathBuf>,

    /// Exact file path at which to save the downloaded asset (enables renaming).
    /// If the path already exists as a file it will be overwritten.
    /// Must not point to an existing directory.
    /// Parent directories are created automatically if they do not exist.
    /// Mutually exclusive with --dir.
    #[arg(short = 'O', long, conflicts_with = "dir")]
    output: Option<std::path::PathBuf>,

    /// Extract the downloaded archive to the destination directory.
    /// Supports .tar.gz and .tgz formats. The archive is not saved to disk.
    /// Mutually exclusive with --output.
    #[arg(short = 'x', long, conflicts_with = "output")]
    extract: bool,

    /// Extract a single file or directory entry from the archive by its internal path
    /// (e.g. `bin/mytool` or `share/config`). Supports .tar.gz and .tgz formats.
    /// The archive is not saved to disk. Use --output to rename the extracted entry
    /// or --dir to choose the destination directory.
    /// Mutually exclusive with --extract.
    #[arg(short = 'X', long, conflicts_with = "extract")]
    extract_entry: Option<String>,
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    let api_url = to_api_url(&cli.url)?;
    let release = fetch_release(&api_url)?;
    let asset = select_asset(&release.assets, &cli.pattern)?;
    let reader = fetch_asset(asset)?;

    if let Some(ref entry) = cli.extract_entry {
        if !is_extractable(&asset.name) {
            return Err(AppError::UnsupportedFormat(asset.name.clone()));
        }
        let norm = normalize_entry_path(entry);
        let entry_basename = Path::new(norm)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(norm);
        let dest = resolve_output_path(entry_basename, cli.dir.as_deref(), cli.output.as_deref())?;
        extract_archive_entry(reader, entry, &dest)?;
        println!("Extracted to: {}", dest.display());
        return Ok(());
    }

    if cli.extract {
        if !is_extractable(&asset.name) {
            return Err(AppError::UnsupportedFormat(asset.name.clone()));
        }
        let dest_dir = cli.dir.as_deref().unwrap_or(Path::new("."));
        extract_archive(reader, dest_dir)?;
        println!("Extracted to: {}", dest_dir.display());
        return Ok(());
    }

    let dest = resolve_output_path(&asset.name, cli.dir.as_deref(), cli.output.as_deref())?;
    save_to_file(reader, &dest)?;
    println!("Downloaded: {}", dest.display());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    #[test]
    fn test_dir_and_output_mutually_exclusive() {
        let result = Cli::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--dir",
            "/tmp",
            "--output",
            "/tmp/file.bin",
        ]);
        assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn test_extract_and_output_mutually_exclusive() {
        let result = Cli::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--output",
            "/tmp/file.bin",
            "--extract",
        ]);
        assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn test_extract_entry_and_extract_mutually_exclusive() {
        let result = Cli::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--extract-entry",
            "bin/tool",
            "--extract",
        ]);
        assert_eq!(result.unwrap_err().kind(), ErrorKind::ArgumentConflict);
    }
}
