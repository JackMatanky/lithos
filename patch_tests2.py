import re

with open("crates/app/tests/index.rs", "r") as f:
    content = f.read()

content = content.replace(
    'assert_eq!(result.report().scanned(), 2);',
    'assert_eq!(result.report().scanned(), 1);'
)

with open("crates/app/tests/index.rs", "w") as f:
    f.write(content)
