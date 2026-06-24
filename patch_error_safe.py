import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# 1. Add imports
content = content.replace(
    'use trace_settings::DiscoveryError;',
    'use trace_settings::DiscoveryError;\nuse std::path::PathBuf;\nuse trace_indexer::{IndexerError, ScannerError, error::IndexerRepositoryError};'
)

# 2. Add Index variant to CliError
content = content.replace(
    '    InvalidPath(String),',
    '    InvalidPath(String),\n\n    /// Error during the index operation.\n    #[error(transparent)]\n    Index(#[from] IndexCommandError),'
)

# 3. Add IndexCommandError and impl
index_err_code = """
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum IndexCommandError {
    #[error("{} does not exist", .path.display())]
    #[diagnostic(help("Provide a valid path, or omit --path to index the entire vault"))]
    ScanPathNotFound { path: PathBuf },

    #[error("cannot read {}: permission denied", .path.display())]
    #[diagnostic(help("Grant read permission: chmod +r {}", .path.display()))]
    ScanAccessDenied { path: PathBuf },

    #[error("index database error: {detail}")]
    #[diagnostic(help("Run `traces index --rebuild` to recreate the database"))]
    StorageFailure { detail: String },

    #[error("I/O error reading {}: {detail}", .path.display())]
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
            IndexerError::Repository(other) => {
                IndexCommandError::StorageFailure { detail: other.to_string() }
            }
            _ => IndexCommandError::StorageFailure { detail: err.to_string() }
        }
    }
}
"""
content = re.sub(r'}\n\nimpl CliError {', r'}\n' + index_err_code + '\nimpl CliError {', content)

# 4. Change exit_code logic
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
