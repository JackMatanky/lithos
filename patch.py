import re

with open("crates/cli/src/commands/index.rs", "r") as f:
    content = f.read()

new_code = """
    let _result =
        app_run_index(&vault_root, &cache_dir, &cmd).map_err(CliError::from)?;

    format_output(&_result, _format, &mut _out)?;

    Ok(())
}

fn format_output(
    result: &trace_app::index::IndexResult,
    format: OutputFormat,
    mut out: impl std::io::Write,
) -> Result<(), CliError> {
    let report = result.report();
    match format {
        OutputFormat::Human => {
            writeln!(out, "{:>9}: {}", "scanned", report.scanned())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
            writeln!(out, "{:>9}: {}", "new", report.new_count())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
            writeln!(out, "{:>9}: {}", "fresh", report.fresh_count())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
            writeln!(out, "{:>9}: {}", "stale", report.stale_count())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
            writeln!(out, "{:>9}: {}", "deleted", report.deleted_count())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
            writeln!(out, "{:>9}: {}", "failed", report.failures().len())
                .map_err(|source| CliError::Write { stream: "stdout", source })?;
        }
        OutputFormat::Json => {
            writeln!(
                out,
                r#"{{"scanned":{},"new":{},"fresh":{},"stale":{},"deleted":{},"failed":{}}}"#,
                report.scanned(),
                report.new_count(),
                report.fresh_count(),
                report.stale_count(),
                report.deleted_count(),
                report.failures().len()
            )
            .map_err(|source| CliError::Write { stream: "stdout", source })?;
        }
    }
    Ok(())
}
"""

content = re.sub(r'    let _result =\n        app_run_index\(&vault_root, &cache_dir, &cmd\).map_err\(CliError::from\)\?;\n\n    // Formatting for CLI output is pending in Cycle 5 \(Output formatting\).\n    // The test in this cycle only requires mapping arguments to models.\n    Ok\(\)\n\}', new_code.strip(), content)

with open("crates/cli/src/commands/index.rs", "w") as f:
    f.write(content)
