import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

content = content.replace(
    '#![allow(unused_assignments)]',
    '#![allow(unused_assignments, reason = "miette macro generates unused bindings")]'
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
