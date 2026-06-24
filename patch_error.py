import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# Add imports
content = re.sub(
    r'use trace_settings::DiscoveryError;',
    'use trace_settings::DiscoveryError;\nuse std::path::PathBuf;\nuse trace_indexer::{IndexerError, ScannerError, error::IndexerRepositoryError};',
    content
)

# Add Index variant to CliError
content = re.sub(
    r'    InvalidPath\(String\),',
    '    InvalidPath(String),\n\n    /// Error during the index operation.\n    #[error(transparent)]\n    Index(#[from] IndexCommandError),',
    content
)

# Add IndexCommandError
index_err_code = """
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum IndexCommandError {
    #[error("{path} does not exist")]
    #[diagnostic(help("Provide a valid path, or omit --path to index the entire vault"))]
    ScanPathNotFound { path: PathBuf },

    #[error("cannot read {path}: permission denied")]
    #[diagnostic(help("Grant read permission: chmod +r {path}"))]
    ScanAccessDenied { path: PathBuf },

    #[error("index database error: {detail}")]
    #[diagnostic(help("Run `traces index --rebuild` to recreate the database"))]
    StorageFailure { detail: String },

    #[error("I/O error reading {path}: {detail}")]
    #[diagnostic(help("Check disk space and filesystem health, then retry"))]
    ScanIoError { path: PathBuf, detail: String },
}

impl From<IndexerError> for IndexCommandError {
    fn from(err: IndexerError) -> Self {
        match err {
            IndexerError::Path(e) => IndexCommandError::ScanPathNotFound {
                path: PathBuf::from(e.to_string()),
            },
            IndexerError::Scanner(ScannerError::Traversal { path, source }) => {
                match source.kind() {
                    std::io::ErrorKind::NotFound => IndexCommandError::ScanPathNotFound { path },
                    std::io::ErrorKind::PermissionDenied => IndexCommandError::ScanAccessDenied { path },
                    _ => IndexCommandError::ScanIoError { path, detail: source.to_string() },
                }
            }
            IndexerError::Repository(IndexerRepositoryError::Storage(e)) => {
                IndexCommandError::StorageFailure { detail: e.to_string() }
            }
            IndexerError::Repository(IndexerRepositoryError::DuplicatePath(p)) => {
                IndexCommandError::StorageFailure { detail: format!("duplicate path: {}", p.as_str()) }
            }
        }
    }
}
"""
content = re.sub(r'}\n\nimpl CliError {', r'}\n' + index_err_code + '\nimpl CliError {', content)

# Change exit_code logic
old_exit_logic = """
            Self::Bootstrap(AppError::Indexer(_))
            | Self::Write {
                ..
            } => 3,
"""
new_exit_logic = """
            Self::Bootstrap(AppError::Indexer(_)) => unreachable!("Indexer error should be mapped to IndexCommandError"),
            Self::Write { .. } => 3,
            Self::Index(err) => match err {
                IndexCommandError::ScanPathNotFound { .. } => 2,
                IndexCommandError::ScanAccessDenied { .. } => 3,
                IndexCommandError::StorageFailure { .. } => 2,
                IndexCommandError::ScanIoError { .. } => 3,
            },
"""
content = content.replace(old_exit_logic.strip(), new_exit_logic.strip())

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
