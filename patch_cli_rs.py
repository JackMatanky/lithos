import re

with open("crates/cli/src/cli.rs", "r") as f:
    content = f.read()

content = content.replace(
    '/// Arguments for the `index` subcommand.\n#[derive(Debug, Args, PartialEq, Eq)]\npub(crate) struct IndexArgs {',
    '/// Arguments for the `index` subcommand.\n///\n/// EXAMPLES:\n///   $ traces index\n///   $ traces index --rebuild\n///   $ traces index --path templates/ --format json\n#[derive(Debug, Args, PartialEq, Eq)]\npub(crate) struct IndexArgs {'
)

with open("crates/cli/src/cli.rs", "w") as f:
    f.write(content)
