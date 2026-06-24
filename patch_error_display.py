import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

content = content.replace(
    '#[diagnostic(help("Provide a valid path, or omit --path to index the entire vault"))]\n    ScanPathNotFound { path: PathBuf },',
    '#[diagnostic(help("Provide a valid path, or omit --path to index the entire vault"))]\n    ScanPathNotFound { path: String },'
)

content = content.replace(
    '#[diagnostic(help("Grant read permission: chmod +r {path}"))]\n    ScanAccessDenied { path: PathBuf },',
    '#[diagnostic(help("Grant read permission: chmod +r {path}"))]\n    ScanAccessDenied { path: String },'
)

content = content.replace(
    '#[diagnostic(help("Check disk space and filesystem health, then retry"))]\n    ScanIoError { path: PathBuf, detail: String },',
    '#[diagnostic(help("Check disk space and filesystem health, then retry"))]\n    ScanIoError { path: String, detail: String },'
)

content = content.replace(
    'IndexerError::Path(e) => IndexCommandError::ScanPathNotFound {\n                path: PathBuf::from(e.to_string()),\n            }',
    'IndexerError::Path(e) => IndexCommandError::ScanPathNotFound {\n                path: e.to_string(),\n            }'
)

content = content.replace(
    'std::io::ErrorKind::NotFound => IndexCommandError::ScanPathNotFound { path },',
    'std::io::ErrorKind::NotFound => IndexCommandError::ScanPathNotFound { path: path.display().to_string() },'
)

content = content.replace(
    'std::io::ErrorKind::PermissionDenied => IndexCommandError::ScanAccessDenied { path },',
    'std::io::ErrorKind::PermissionDenied => IndexCommandError::ScanAccessDenied { path: path.display().to_string() },'
)

content = content.replace(
    '_ => IndexCommandError::ScanIoError { path, detail: source.to_string() },',
    '_ => IndexCommandError::ScanIoError { path: path.display().to_string(), detail: source.to_string() },'
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
