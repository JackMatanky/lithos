import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# Fix import
content = content.replace(
    'error::IndexerRepositoryError',
    'IndexerRepositoryError'
)

# Fix thiserror formatting to avoid unused variable warning
content = content.replace(
    '#[error("{} does not exist", .path.display())]',
    '#[error("{} does not exist", path.display())]'
)
content = content.replace(
    '#[error("cannot read {}: permission denied", .path.display())]',
    '#[error("cannot read {}: permission denied", path.display())]'
)
content = content.replace(
    '#[diagnostic(help("Grant read permission: chmod +r {}", .path.display()))]',
    '#[diagnostic(help("Grant read permission: chmod +r {}", path.display()))]'
)
content = content.replace(
    '#[error("I/O error reading {}: {detail}", .path.display())]',
    '#[error("I/O error reading {}: {detail}", path.display())]'
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
