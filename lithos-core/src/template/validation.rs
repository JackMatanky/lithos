use super::error::TemplateError;

/// Validates template content size.
///
/// # Errors
/// Returns `TemplateError::TemplateContentTooLarge` if content exceeds 1MB.
#[inline]
pub fn validate_content(content: &str) -> Result<(), TemplateError> {
    if content.len() > 1024 * 1024 {
        return Err(TemplateError::TemplateContentTooLarge(
            content.len(),
            1024 * 1024,
        ));
    }
    Ok(())
}

/// Validates that template content has balanced placeholders.
/// (Basic structure validation as required by story).
///
/// # Errors
/// Returns `TemplateError::ValidationFailed` if placeholders are unbalanced.
#[inline]
pub fn validate_structure(
    content: &str,
    prefix: &str,
    suffix: &str,
) -> Result<(), TemplateError> {
    let open_count = content.matches(prefix).count();
    let close_count = content.matches(suffix).count();

    if open_count != close_count {
        return Err(TemplateError::ValidationFailed(format!(
            "Unbalanced template placeholders: {open_count} opening, \
             {close_count} closing"
        )));
    }
    Ok(())
}
