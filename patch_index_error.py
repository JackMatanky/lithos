import re

with open("crates/cli/src/commands/index.rs", "r") as f:
    content = f.read()

new_err_mapping = """
    let _result = app_run_index(&vault_root, &cache_dir, &cmd).map_err(|e| match e {
        trace_app::error::AppError::Indexer(idx_err) => {
            CliError::Index(crate::error::IndexCommandError::from(idx_err))
        }
        other => CliError::Bootstrap(other),
    })?;
"""
content = re.sub(
    r'    let _result =\n        app_run_index\(&vault_root, &cache_dir, &cmd\).map_err\(CliError::from\)\?;',
    new_err_mapping.strip(),
    content
)

with open("crates/cli/src/commands/index.rs", "w") as f:
    f.write(content)
