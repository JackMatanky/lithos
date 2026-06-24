import re

with open("crates/app/tests/index.rs", "r") as f:
    content = f.read()

content = content.replace(
    'let mut opts = IndexOptions::default();\n    opts.set_rebuild(true);',
    'let opts = IndexOptions::new(true, false);'
)

content = content.replace(
    'let mut opts = IndexOptions::default();\n    opts.set_dry_run(true);',
    'let opts = IndexOptions::new(false, true);'
)

with open("crates/app/tests/index.rs", "w") as f:
    f.write(content)
