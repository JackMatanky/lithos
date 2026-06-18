//! Shared output formatting for CLI command handlers.
//!
//! All structured output uses [`serde_json`] for JSON serialisation and
//! plain `writeln!` for human-readable output.

use std::io::Write;

use crate::error::CliError;

/// Write a value as JSON to `out`, mapping I/O errors to [`CliError::Write`].
///
/// # Errors
///
/// Returns [`CliError::Write`] if serialisation or the underlying write fails.
pub(crate) fn write_json<W: Write, T: serde::Serialize>(
    out: &mut W,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer(out, value).map_err(|e| CliError::Write {
        stream: "stdout",
        source: e.into(),
    })
}

/// Write a value as JSON followed by a newline to `out`.
///
/// # Errors
///
/// Returns [`CliError::Write`] if serialisation, the JSON write, or the
/// trailing newline write fails.
pub(crate) fn write_json_line<W: Write, T: serde::Serialize>(
    out: &mut W,
    value: &T,
) -> Result<(), CliError> {
    write_json(out, value)?;
    writeln!(out).map_err(|e| CliError::Write {
        stream: "stdout",
        source: e,
    })
}

/// Map a `writeln!` error on stdout to [`CliError::Write`].
pub(crate) fn stdout_err(e: std::io::Error) -> CliError {
    CliError::Write {
        stream: "stdout",
        source: e,
    }
}

/// Map a `writeln!` error on stderr to [`CliError::Write`].
pub(crate) fn stderr_err(e: std::io::Error) -> CliError {
    CliError::Write {
        stream: "stderr",
        source: e,
    }
}
