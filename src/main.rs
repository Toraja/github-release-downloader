use clap::{CommandFactory, Parser};
use regex::Regex;
use url::Url;

mod archive;
mod error;
mod github;

use archive::{Destination, extract_archive, is_extractable, save_to_file};
use error::AppError;
use github::{fetch_asset, fetch_release, select_asset, to_api_url};

/// Download a release asset from a GitHub repository.
///
/// Fetches the latest release from the given GitHub repository and downloads
/// the single asset whose name matches PATTERN (a regular expression).
///
/// Authentication: set the GITHUB_TOKEN environment variable to use an
/// authenticated request and avoid the 60 req/hr unauthenticated rate limit.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
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
    /// Use --archive-entry to narrow extraction to a single entry.
    #[arg(short = 'x', long)]
    extract: bool,

    /// Narrow --extract to a single file or directory entry by its internal archive path
    /// (e.g. `bin/mytool` or `share/config`). Supports .tar.gz and .tgz formats.
    /// The archive is not saved to disk. Use --output to rename the extracted entry
    /// or --dir to choose the destination directory.
    /// Requires --extract.
    #[arg(short = 'X', long, requires = "extract")]
    archive_entry: Option<String>,
}

impl Args {
    /// Post-parse validation for rules Clap cannot express declaratively.
    ///
    /// Rejects `--output` together with `--extract` when `--archive-entry` is absent:
    /// whole-archive extraction produces a directory of files, not a single renameable path.
    /// When `--archive-entry` is present, `--output` renames the single extracted entry
    /// and is allowed.
    ///
    /// Returns a `clap::Error` so callers can handle it (e.g. exit or assert in tests).
    fn try_validate(&self) -> Result<(), clap::Error> {
        if self.extract && self.archive_entry.is_none() && self.output.is_some() {
            return Err(Args::command().error(
                clap::error::ErrorKind::ArgumentConflict,
                "--output cannot be used when extracting a whole archive; use --dir instead",
            ));
        }
        Ok(())
    }

    fn validate(&self) {
        self.try_validate().unwrap_or_else(|e| e.exit());
    }
}

fn run() -> Result<(), AppError> {
    let args = Args::parse();
    args.validate();

    let api_url = to_api_url(&args.url)?;
    let release = fetch_release(&api_url)?;
    let asset = select_asset(&release.assets, &args.pattern)?;

    if args.extract && !is_extractable(&asset.name) {
        return Err(AppError::UnsupportedFormat(asset.name.clone()));
    }

    let reader = fetch_asset(asset)?;

    if args.extract {
        let dest = Destination::resolve(args.dir.as_deref(), args.output.as_deref())?;
        let landing = extract_archive(reader, args.archive_entry.as_deref(), dest)?;
        println!("Extracted to: {}", landing.display());
        return Ok(());
    }

    let dest = Destination::resolve(args.dir.as_deref(), args.output.as_deref())?;
    let landing = save_to_file(reader, &asset.name, dest)?;
    println!("Downloaded: {}", landing.display());

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
        let result = Args::try_parse_from([
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
    fn test_archive_entry_without_extract_rejected() {
        let result = Args::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--archive-entry",
            "bin/tool",
        ]);
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_extract_and_output_whole_archive_rejected() {
        let args = Args::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--output",
            "/tmp/file.bin",
            "--extract",
        ])
        .expect("should parse; conflict is post-parse");
        assert_eq!(
            args.try_validate().unwrap_err().kind(),
            ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn test_extract_with_archive_entry_and_output_accepted() {
        let args = Args::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--extract",
            "--archive-entry",
            "bin/tool",
            "--output",
            "/tmp/mytool",
        ])
        .expect("should parse successfully");
        assert!(args.extract);
        assert_eq!(args.archive_entry.as_deref(), Some("bin/tool"));
        assert!(args.output.is_some());
    }
}
