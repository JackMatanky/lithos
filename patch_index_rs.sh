#!/bin/bash
sed -i '' -e '/\/\/ 4. Format output/,+35c\
    \/\/ 4. Format output\
    format_output(&result, _format, &mut _out)?;\
\
    Ok(())\
}\
\
fn format_output(\
    result: &trace_app::index::IndexResult,\
    format: OutputFormat,\
    mut out: impl Write,\
) -> Result<(), CliError> {\
    let report = result.report();\
    match format {\
        OutputFormat::Human => {\
            writeln!(out, "{:>9}: {}", "scanned", report.scanned())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
            writeln!(out, "{:>9}: {}", "new", report.new_count())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
            writeln!(out, "{:>9}: {}", "fresh", report.fresh_count())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
            writeln!(out, "{:>9}: {}", "stale", report.stale_count())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
            writeln!(out, "{:>9}: {}", "deleted", report.deleted_count())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
            writeln!(out, "{:>9}: {}", "failed", report.failures().len())\
                .map_err(|source| CliError::Write { stream: "stdout", source })?;\
        }\
        OutputFormat::Json => {\
            writeln!(\
                out,\
                r#"{{\\"scanned\\":{},\\"new\\":{},\\"fresh\\":{},\\"stale\\":{},\\"deleted\\":{},\\"failed\\":{}}}"#,\
                report.scanned(),\
                report.new_count(),\
                report.fresh_count(),\
                report.stale_count(),\
                report.deleted_count(),\
                report.failures().len()\
            )\
            .map_err(|source| CliError::Write { stream: "stdout", source })?;\
        }\
    }\
    Ok(())\
}\
' crates/cli/src/commands/index.rs
