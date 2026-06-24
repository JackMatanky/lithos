import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

content = content.replace(
    '#[expect(unused_variables, reason = "miette macro generates unused bindings")]',
    '#[expect(unused_assignments, reason = "miette macro generates unused bindings")]'
)

# Unreachable removal
content = content.replace(
    'Self::Bootstrap(AppError::Indexer(_)) => unreachable!("Indexer error should be mapped to IndexCommandError"),',
    'Self::Bootstrap(AppError::Indexer(_)) => 3,'
)

# Fix match_same_arms
old_match = """
            Self::Index(err) => match err {
                IndexCommandError::ScanPathNotFound { .. } => 2,
                IndexCommandError::ScanAccessDenied { .. } => 3,
                IndexCommandError::StorageFailure { .. } => 2,
                IndexCommandError::ScanIoError { .. } => 3,
            },
"""
new_match = """
            Self::Index(err) => match err {
                IndexCommandError::ScanPathNotFound { .. }
                | IndexCommandError::StorageFailure { .. } => 2,
                IndexCommandError::ScanAccessDenied { .. }
                | IndexCommandError::ScanIoError { .. } => 3,
            },
"""
content = content.replace(old_match.strip(), new_match.strip())

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
