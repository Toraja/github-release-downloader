use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use clap::Parser;
use flate2::read::GzDecoder;
use regex::Regex;
use serde::Deserialize;
use url::Url;

/// Download a release asset from a GitHub repository.
///
/// Fetches the latest release from the given GitHub repository and downloads
/// the single asset whose name matches PATTERN (a regular expression).
///
/// Authentication: set the GITHUB_TOKEN environment variable to use an
/// authenticated request and avoid the 60 req/hr unauthenticated rate limit.
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// GitHub repository URL (e.g., https://github.com/owner/repo)
    url: Url,

    /// Regex pattern to match against asset names (must match exactly one asset)
    pattern: Regex,

    /// Directory in which to save the downloaded asset (original filename is preserved).
    /// Parent directories are created automatically if they do not exist.
    /// Mutually exclusive with --output.
    #[arg(short = 'D', long, conflicts_with = "output")]
    dir: Option<PathBuf>,

    /// Exact file path at which to save the downloaded asset (enables renaming).
    /// If the path already exists as a file it will be overwritten.
    /// Must not point to an existing directory.
    /// Parent directories are created automatically if they do not exist.
    /// Mutually exclusive with --dir.
    #[arg(short = 'O', long, conflicts_with = "dir")]
    output: Option<PathBuf>,

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

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

fn to_api_url(url: &Url) -> Result<Url, String> {
    if url.host_str() != Some("github.com") {
        return Err(format!(
            "Invalid URL: host must be github.com, got {}",
            url.host_str().unwrap_or("<none>")
        ));
    }

    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).collect())
        .unwrap_or_default();

    if segments.len() < 2 {
        return Err(format!(
            "Invalid URL: expected owner/repo path segments, got {}",
            url.path()
        ));
    }

    let owner = segments[0];
    let repo = segments[1];

    Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    ))
    .map_err(|e| e.to_string())
}

fn fetch_release(api_url: &Url) -> Result<Release, String> {
    let mut req = ureq::get(api_url.as_str())
        .header("User-Agent", "github-latest-release-downloader")
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    let mut response = req
        .call()
        .map_err(|e| format!("API request failed: {}", e))?;

    response
        .body_mut()
        .read_json::<Release>()
        .map_err(|e| format!("Failed to parse release JSON: {}", e))
}

fn select_asset<'a>(assets: &'a [Asset], pattern: &Regex) -> Result<&'a Asset, String> {
    let matches: Vec<&Asset> = assets
        .iter()
        .filter(|a| pattern.is_match(&a.name))
        .collect();

    match matches.len() {
        0 => {
            let available: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
            Err(format!(
                "No assets matched pattern '{}'. Available assets:\n  {}",
                pattern,
                available.join("\n  ")
            ))
        }
        1 => Ok(matches[0]),
        _ => {
            let matched: Vec<&str> = matches.iter().map(|a| a.name.as_str()).collect();
            Err(format!(
                "Pattern '{}' matched multiple assets — refine your pattern:\n  {}",
                pattern,
                matched.join("\n  ")
            ))
        }
    }
}

fn is_extractable(asset_name: &str) -> bool {
    asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz")
}

fn resolve_output_path(
    asset_name: &str,
    dir: Option<&Path>,
    output: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(output) = output {
        if output.is_dir() {
            return Err(format!(
                "--output path '{}' is an existing directory; use --dir to save into a directory",
                output.display()
            ));
        }
        return Ok(output.to_path_buf());
    }

    let base = dir.unwrap_or_else(|| Path::new("."));
    Ok(base.join(asset_name))
}

fn download_asset(asset: &Asset, dest: &Path) -> Result<(), String> {
    let mut response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", "github-latest-release-downloader")
        .call()
        .map_err(|e| format!("Download request failed: {}", e))?;

    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let mut file = File::create(dest)
        .map_err(|e| format!("Failed to create file '{}': {}", dest.display(), e))?;

    let mut reader = response.body_mut().as_reader();
    io::copy(&mut reader, &mut file)
        .map_err(|e| format!("Failed to write '{}': {}", dest.display(), e))?;

    Ok(())
}

fn extract_asset(asset: &Asset, dest_dir: &Path) -> Result<(), String> {
    let mut response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", "github-latest-release-downloader")
        .call()
        .map_err(|e| format!("Download request failed: {}", e))?;

    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create directory '{}': {}", dest_dir.display(), e))?;

    let reader = response.body_mut().as_reader();
    unpack_tar_gz(reader, dest_dir)
}

fn unpack_tar_gz<R: Read>(reader: R, dest_dir: &Path) -> Result<(), String> {
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(dest_dir)
        .map_err(|e| format!("Failed to extract archive: {}", e))
}

/// Strip a leading `./` and a trailing `/` from an archive entry path.
fn normalize_entry_path(s: &str) -> &str {
    let s = s.strip_prefix("./").unwrap_or(s);
    s.trim_end_matches('/')
}

/// Core logic for `--extract-entry`: iterate the tar.gz stream and extract the
/// matching file or directory entry to `dest`.
///
/// - File entry: exact normalised-path match → written directly to `dest`.
/// - Directory entry: prefix match → contents recreated under `dest/`.
/// - Symlink as the specified entry → error.
/// - Symlink as a child during directory extraction → warning + skip.
/// - No match → error listing top-level archive entries.
fn extract_entry_from_reader<R: Read>(reader: R, entry: &str, dest: &Path) -> Result<(), String> {
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);

    let norm_entry = normalize_entry_path(entry).to_string();
    let dir_prefix = format!("{}/", norm_entry);

    let mut matched = false;
    let mut top_level: Vec<String> = Vec::new();

    for entry_result in archive
        .entries()
        .map_err(|e| format!("Failed to read archive: {}", e))?
    {
        let mut tar_entry =
            entry_result.map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let path_owned = tar_entry
            .path()
            .map_err(|e| format!("Failed to read entry path: {}", e))?
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
                return Err(format!(
                    "Entry '{}' is a symlink; symlink extraction is not supported",
                    norm_entry
                ));
            }
            if let Some(parent) = dest.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
            tar_entry
                .unpack(dest)
                .map_err(|e| format!("Failed to extract entry '{}': {}", norm_path, e))?;
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
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
            tar_entry
                .unpack(&dest_path)
                .map_err(|e| format!("Failed to extract entry '{}': {}", norm_path, e))?;
            matched = true;
        }
    }

    if !matched {
        top_level.sort();
        return Err(format!(
            "Entry '{}' not found in archive. Top-level entries:\n  {}",
            norm_entry,
            top_level.join("\n  ")
        ));
    }

    Ok(())
}

fn extract_entry(asset: &Asset, entry: &str, dest: &Path) -> Result<(), String> {
    let mut response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", "github-latest-release-downloader")
        .call()
        .map_err(|e| format!("Download request failed: {}", e))?;

    let reader = response.body_mut().as_reader();
    extract_entry_from_reader(reader, entry, dest)
}

fn main() {
    let cli = Cli::parse();

    let api_url = match to_api_url(&cli.url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let release = match fetch_release(&api_url) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let asset = match select_asset(&release.assets, &cli.pattern) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(ref entry) = cli.extract_entry {
        if !is_extractable(&asset.name) {
            eprintln!(
                "Error: --extract-entry requires a supported archive format (.tar.gz or .tgz), but '{}' is not supported",
                asset.name
            );
            std::process::exit(1);
        }
        let norm = normalize_entry_path(entry);
        let entry_basename = Path::new(norm)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(norm);
        let dest =
            match resolve_output_path(entry_basename, cli.dir.as_deref(), cli.output.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
        if let Err(e) = extract_entry(asset, entry, &dest) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("Extracted to: {}", dest.display());
        return;
    }

    if cli.extract && !is_extractable(&asset.name) {
        eprintln!(
            "Error: --extract requires a supported archive format (.tar.gz or .tgz), but '{}' is not supported",
            asset.name
        );
        std::process::exit(1);
    }

    if cli.extract {
        let dest_dir = cli.dir.as_deref().unwrap_or(Path::new("."));
        if let Err(e) = extract_asset(asset, dest_dir) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("Extracted to: {}", dest_dir.display());
        return;
    }

    let dest = match resolve_output_path(&asset.name, cli.dir.as_deref(), cli.output.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = download_asset(asset, &dest) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Downloaded: {}", dest.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_asset(name: &str) -> Asset {
        Asset {
            name: name.to_string(),
            browser_download_url: format!("https://example.com/{}", name),
        }
    }

    #[test]
    #[allow(clippy::invalid_regex)]
    fn test_invalid_regex() {
        assert!(Regex::new("[invalid").is_err());
    }

    #[test]
    fn test_no_matching_assets() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new("windows").unwrap();
        let err = select_asset(&assets, &pattern).unwrap_err();
        assert!(err.contains("No assets matched"));
        assert!(err.contains("gh_2.40.1_linux_amd64.tar.gz"));
        assert!(err.contains("gh_2.40.1_darwin_amd64.tar.gz"));
    }

    #[test]
    fn test_multiple_matching_assets() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_linux_arm64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new("linux").unwrap();
        let err = select_asset(&assets, &pattern).unwrap_err();
        assert!(err.contains("matched multiple assets"));
        assert!(err.contains("gh_2.40.1_linux_amd64.tar.gz"));
        assert!(err.contains("gh_2.40.1_linux_arm64.tar.gz"));
        assert!(!err.contains("darwin"));
    }

    #[test]
    fn test_single_matching_asset() {
        let assets = vec![
            make_asset("gh_2.40.1_linux_amd64.tar.gz"),
            make_asset("gh_2.40.1_darwin_amd64.tar.gz"),
        ];
        let pattern = Regex::new(r"linux_amd64").unwrap();
        let asset = select_asset(&assets, &pattern).unwrap();
        assert_eq!(asset.name, "gh_2.40.1_linux_amd64.tar.gz");
    }

    #[test]
    fn test_standard_url() {
        let url = Url::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(
            to_api_url(&url),
            Ok(Url::parse("https://api.github.com/repos/owner/repo/releases/latest").unwrap())
        );
    }

    #[test]
    fn test_trailing_slash_url() {
        let url = Url::parse("https://github.com/owner/repo/").unwrap();
        assert_eq!(
            to_api_url(&url),
            Ok(Url::parse("https://api.github.com/repos/owner/repo/releases/latest").unwrap())
        );
    }

    #[test]
    fn test_non_github_domain() {
        let url = Url::parse("https://gitlab.com/owner/repo").unwrap();
        assert!(to_api_url(&url).is_err());
    }

    #[test]
    fn test_missing_repo_segment() {
        let url = Url::parse("https://github.com/owner").unwrap();
        assert!(to_api_url(&url).is_err());
    }

    // --- resolve_output_path tests ---

    #[test]
    fn test_resolve_no_flags_uses_current_dir() {
        let path = resolve_output_path("asset.tar.gz", None, None).unwrap();
        assert_eq!(path, Path::new(".").join("asset.tar.gz"));
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
        let path =
            resolve_output_path("asset.tar.gz", None, Some(Path::new("/tmp/renamed.bin"))).unwrap();
        assert_eq!(path, Path::new("/tmp/renamed.bin"));
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
        assert!(err.contains("existing directory"));
    }

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
        assert!(result.is_err());
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
    fn test_extract_and_output_mutually_exclusive() {
        let result = Cli::try_parse_from([
            "prog",
            "https://github.com/owner/repo",
            "pattern",
            "--output",
            "/tmp/file.bin",
            "--extract",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpack_tar_gz_extracts_files() {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        // Create a source file to pack
        let src_dir = tempfile::tempdir().unwrap();
        let src_file = src_dir.path().join("hello.txt");
        std::fs::write(&src_file, "hello from tarball").unwrap();

        // Build a .tar.gz in memory
        let gz = GzEncoder::new(Vec::new(), Compression::default());
        let mut archive = tar::Builder::new(gz);
        archive
            .append_path_with_name(&src_file, "hello.txt")
            .unwrap();
        let gz = archive.into_inner().unwrap();
        let gz_data = gz.finish().unwrap();

        // Extract and verify
        let dest_dir = tempfile::tempdir().unwrap();
        unpack_tar_gz(gz_data.as_slice(), dest_dir.path()).unwrap();

        let extracted = dest_dir.path().join("hello.txt");
        assert!(extracted.exists(), "expected hello.txt to be extracted");
        assert_eq!(
            std::fs::read_to_string(&extracted).unwrap(),
            "hello from tarball"
        );
    }

    // ── extract_entry_from_reader helpers ────────────────────────────────────

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
                    // Symlink entry; target is arbitrary for these tests.
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

    // ── mutual exclusion ─────────────────────────────────────────────────────

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
        assert!(result.is_err());
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
        extract_entry_from_reader(data.as_slice(), "bin/tool", &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "binary content");
    }

    // ── file entry → --dir ───────────────────────────────────────────────────

    #[test]
    fn test_extract_entry_file_with_dir() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("binary"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("tool"); // as resolved by resolve_output_path with --dir
        extract_entry_from_reader(data.as_slice(), "bin/tool", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "binary");
    }

    // ── file entry → --output (rename) ──────────────────────────────────────

    #[test]
    fn test_extract_entry_file_with_output_rename() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("renamed content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("mytool");
        extract_entry_from_reader(data.as_slice(), "bin/tool", &dest).unwrap();
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
        extract_entry_from_reader(data.as_slice(), "share/config", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("a.conf")).unwrap(), "aaa");
        assert_eq!(std::fs::read_to_string(dest.join("b.conf")).unwrap(), "bbb");
        assert!(!tmp.path().join("config").join("../other").exists());
    }

    // ── directory entry → --dir ──────────────────────────────────────────────

    #[test]
    fn test_extract_entry_dir_with_dir_flag() {
        let data = make_tar_gz_with_entries(&[("pkg/lib/x.so", Some("lib"))]);
        let tmp = tempfile::tempdir().unwrap();
        // resolve_output_path("lib", Some(tmp), None) → tmp/lib
        let dest = tmp.path().join("lib");
        extract_entry_from_reader(data.as_slice(), "pkg/lib", &dest).unwrap();
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
        extract_entry_from_reader(data.as_slice(), "share/config", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("a.conf")).unwrap(), "aaa");
        assert_eq!(
            std::fs::read_to_string(dest.join("sub/b.conf")).unwrap(),
            "bbb"
        );
    }

    // ── directory entry → merges into existing dest, overlapping file overwritten

    #[test]
    fn test_extract_entry_dir_merges_into_existing_dest() {
        // Archive: mydir/foo/bar + mydir/foo/baz
        let data = make_tar_gz_with_entries(&[
            ("mydir/foo/bar", Some("from archive")),
            ("mydir/foo/baz", Some("also from archive")),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("out");
        // Dest already exists: foo/bar (will be overwritten) and foo/quux (must survive).
        std::fs::create_dir_all(dest.join("foo")).unwrap();
        std::fs::write(dest.join("foo/bar"), "original").unwrap();
        std::fs::write(dest.join("foo/quux"), "also original").unwrap();

        extract_entry_from_reader(data.as_slice(), "mydir", &dest).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.join("foo/bar")).unwrap(),
            "from archive",
            "bar should be overwritten by the archive"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("foo/baz")).unwrap(),
            "also from archive",
            "baz should be newly created from the archive"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("foo/quux")).unwrap(),
            "also original",
            "quux should be left untouched"
        );
    }

    // ── entry not found → error lists top-level entries ─────────────────────

    #[test]
    fn test_extract_entry_not_found_lists_top_level() {
        let data =
            make_tar_gz_with_entries(&[("bin/tool", Some("x")), ("share/man/tool.1", Some("y"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("out");
        let err = extract_entry_from_reader(data.as_slice(), "no/such/path", &dest).unwrap_err();
        assert!(err.contains("not found"), "expected 'not found' in: {err}");
        assert!(err.contains("bin"), "expected top-level 'bin' in: {err}");
        assert!(
            err.contains("share"),
            "expected top-level 'share' in: {err}"
        );
    }

    // ── unsupported archive format ───────────────────────────────────────────

    #[test]
    fn test_extract_entry_unsupported_format() {
        assert!(!is_extractable("tool-v1.0.zip"));
        assert!(!is_extractable("tool-v1.0.tar.bz2"));
    }

    // ── directly specified symlink entry → error ─────────────────────────────

    #[test]
    fn test_extract_entry_symlink_direct_returns_error() {
        let data = make_tar_gz_with_entries(&[
            ("bin/tool", None), // symlink
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("tool");
        let err = extract_entry_from_reader(data.as_slice(), "bin/tool", &dest).unwrap_err();
        assert!(
            err.contains("symlink"),
            "expected 'symlink' in error: {err}"
        );
    }

    // ── child symlinks during directory extraction → skip + warn ────────────

    #[test]
    fn test_extract_entry_dir_child_symlinks_skipped() {
        let data = make_tar_gz_with_entries(&[
            ("pkg/real.txt", Some("real")),
            ("pkg/link.txt", None), // symlink — should be skipped
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("pkg");
        // Should succeed (not error) and extract only the regular file.
        extract_entry_from_reader(data.as_slice(), "pkg", &dest).unwrap();
        assert!(dest.join("real.txt").exists());
        assert!(!dest.join("link.txt").exists());
    }

    // ── parent directories created automatically ─────────────────────────────

    #[test]
    fn test_extract_entry_creates_parent_dirs() {
        let data = make_tar_gz_with_entries(&[("bin/tool", Some("content"))]);
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("new/nested/dir/tool");
        extract_entry_from_reader(data.as_slice(), "bin/tool", &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "content");
    }
}
