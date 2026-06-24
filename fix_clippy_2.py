import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

content = content.replace(
    '#![allow(unused_assignments)]\n',
    ''
)
content = '#![allow(unused_assignments)]\n' + content

content = content.replace(
    '#[expect(unused_assignments, reason = "miette macro generates unused bindings")]\n',
    ''
)

old_arms = """
            Self::Bootstrap(AppError::Indexer(_)) => 3,
            Self::Write { .. } => 3,
"""
new_arms = """
            Self::Bootstrap(AppError::Indexer(_)) | Self::Write { .. } => 3,
"""
content = content.replace(old_arms.strip(), new_arms.strip())

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
