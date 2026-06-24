import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# 1. Fix import
content = content.replace("error::IndexerRepositoryError", "IndexerRepositoryError")

# 2. Re-do PathBuf to String for all path fields inside IndexCommandError
content = re.sub(
    r'ScanPathNotFound { path: PathBuf }',
    r'ScanPathNotFound { path: String }',
    content
)
content = re.sub(
    r'ScanAccessDenied { path: PathBuf }',
    r'ScanAccessDenied { path: String }',
    content
)
content = re.sub(
    r'ScanIoError { path: PathBuf, detail: String }',
    r'ScanIoError { path: String, detail: String }',
    content
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
