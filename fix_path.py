import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

# Fix IndexCommandError mapping path buf bindings to string.
content = re.sub(
    r'IndexCommandError::ScanPathNotFound {\n                        path,\n                    }',
    r'IndexCommandError::ScanPathNotFound {\n                        path: path.display().to_string(),\n                    }',
    content
)
content = re.sub(
    r'IndexCommandError::ScanAccessDenied {\n                        path,\n                    }',
    r'IndexCommandError::ScanAccessDenied {\n                        path: path.display().to_string(),\n                    }',
    content
)
content = re.sub(
    r'IndexCommandError::ScanIoError {\n                    path,\n                    detail: source.to_string\(\),\n                }',
    r'IndexCommandError::ScanIoError {\n                    path: path.display().to_string(),\n                    detail: source.to_string(),\n                }',
    content
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
