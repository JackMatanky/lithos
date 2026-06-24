import re

with open("crates/cli/src/error.rs", "r") as f:
    content = f.read()

content = content.replace(
    'pub(crate) enum IndexCommandError {',
    '#[expect(unused_variables, reason = "miette macro generates unused bindings")]\npub(crate) enum IndexCommandError {'
)

with open("crates/cli/src/error.rs", "w") as f:
    f.write(content)
