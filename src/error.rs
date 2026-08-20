use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid URL: host must be github.com, got {0}")]
    InvalidHost(String),

    #[error("Invalid URL: expected owner/repo path, got {0}")]
    InvalidPath(String),

    #[error("Internal error: failed to construct API URL: {0}")]
    UrlConstruct(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("Failed to parse release JSON: {0}")]
    JsonParse(String),

    #[error("No assets matched pattern '{pattern}'. Available assets:\n  {available}")]
    NoMatch { pattern: String, available: String },

    #[error("Pattern '{pattern}' matched multiple assets — refine your pattern:\n  {matched}")]
    MultipleMatches { pattern: String, matched: String },

    #[error("--output path '{0}' is an existing directory; use --dir to save into a directory")]
    OutputIsDir(String),

    #[error("--dir path '{0}' is an existing file; it must be a directory")]
    DirIsFile(String),

    #[error("Failed to create directory '{path}': {source}")]
    CreateDir {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to create file '{path}': {source}")]
    CreateFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write '{path}': {source}")]
    WriteFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Download failed: {0}")]
    Download(String),

    #[error(
        "--extract requires a supported archive format (.tar.gz or .tgz), but '{0}' is not supported"
    )]
    UnsupportedFormat(String),

    #[error("Entry '{0}' is a symlink; symlink extraction is not supported")]
    SymlinkEntry(String),

    #[error("Entry '{0}' not found in archive. Top-level entries:\n  {1}")]
    EntryNotFound(String, String),

    #[error("Failed to read archive: {0}")]
    ArchiveRead(String),

    #[error("Failed to extract archive: {0}")]
    ArchiveExtract(String),

    #[error("Asset written to '{path}' but could not set executable bit: {source}")]
    SetPermissions {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("--executable requires a file target, but '{0}' is a directory")]
    ExecutableTargetIsDir(String),
}
