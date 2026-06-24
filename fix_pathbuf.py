import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# Restore back to PathBuf
content = re.sub(
    r'path: e\.to_string\(\)',
    r'path: e.to_string().into()',
    content
)
content = re.sub(
    r'path\.display\(\)\.to_string\(\)',
    r'path',
    content
)

# Fix miette diagnostics to use .display()
content = content.replace(
    '#[diagnostic(help("Grant read permission: chmod +r {path}"))]',
    '#[diagnostic(help("Grant read permission: chmod +r {}", path.display()))]'
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
