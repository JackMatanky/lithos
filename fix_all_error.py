import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# Fix #[error(...)] to use .display() for PathBuf
content = content.replace(
    '#[error("{path} does not exist")]',
    '#[error("{} does not exist", path.display())]'
)
content = content.replace(
    '#[error("cannot read {path}: permission denied")]',
    '#[error("cannot read {}: permission denied", path.display())]'
)
content = content.replace(
    '#[error("I/O error reading {path}: {detail}")]',
    '#[error("I/O error reading {}: {detail}", path.display())]'
)

# Add fallback for _ in IndexerError
fallback = """
            _ => IndexCommandError::StorageFailure {
                detail: err.to_string(),
            },
        }
    }
}
"""
content = content.replace(
    '        }\n    }\n}',
    fallback.strip() + '\n'
)

# For the IndexerRepositoryError match, it is non_exhaustive too!
fallback_repo = """
            IndexerError::Repository(other) => IndexCommandError::StorageFailure {
                detail: other.to_string(),
            },
"""
content = re.sub(
    r'IndexerError::Repository\(\n\s*IndexerRepositoryError::DuplicatePath\(p\),\n\s*\) => IndexCommandError::StorageFailure {\n\s*detail: format!\("duplicate path: {}", p\.as_str\(\)\),\n\s*},',
    r'IndexerError::Repository(IndexerRepositoryError::DuplicatePath(p)) => IndexCommandError::StorageFailure { detail: format!("duplicate path: {}", p.as_str()) },\n' + fallback_repo,
    content,
    flags=re.MULTILINE
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
