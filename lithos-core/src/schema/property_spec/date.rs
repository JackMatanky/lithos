//! Date property validation constraints.

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::error::SchemaError;

/// Date property validation constraints.
#[derive(Debug, Clone, PartialEq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct DateSpec {
    format: Box<str>,
}

impl DateSpec {
    /// Create a validated `DateSpec`.
    ///
    /// Validates that the format string is a valid strftime pattern by probing
    /// it with a known datetime.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidDateFormat` if the format is empty or not
    /// a valid strftime pattern.
    ///
    /// # Panics
    ///
    /// This function will not panic. The `expect` calls are infallible because
    /// the probe datetime is statically known to be valid.
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Probe datetime 2000-01-01 00:00:00 is statically valid"
    )]
    pub fn try_new(format: &str) -> Result<Self, SchemaError> {
        if format.is_empty() {
            return Err(SchemaError::InvalidDateFormat(
                "Format cannot be empty".into(),
            ));
        }

        // Probe: attempt to format a known datetime with this format string
        // Use NaiveDateTime to support both date and datetime format strings
        let probe = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
            .expect("static date should be valid")
            .and_hms_opt(0, 0, 0)
            .expect("static time should be valid");
        let result = probe.format(format).to_string();

        // Verify the format string can parse its own output
        if chrono::NaiveDate::parse_from_str(&result, format).is_err()
            && chrono::NaiveDateTime::parse_from_str(&result, format).is_err()
        {
            return Err(SchemaError::InvalidDateFormat(format!(
                "Format string '{format}' is not a valid strftime pattern"
            )));
        }

        Ok(Self {
            format: format.into(),
        })
    }

    #[inline]
    pub(super) fn validate_str(&self, value: &str) -> Result<(), SchemaError> {
        let is_valid =
            chrono::NaiveDateTime::parse_from_str(value, &self.format).is_ok()
                || chrono::NaiveDate::parse_from_str(value, &self.format)
                    .is_ok();

        if !is_valid {
            return Err(SchemaError::InvalidDateFormat(format!(
                "Value {value} does not match format {}",
                self.format
            )));
        }
        Ok(())
    }

    /// Apply overrides from a raw date spec.
    ///
    /// If the override format is `None`, the base format is preserved.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidDateFormat` if the override format is
    /// invalid.
    #[inline]
    pub fn apply_overrides(
        self,
        overrides: &crate::schema::raw::RawDateSpec,
    ) -> Result<Self, SchemaError> {
        if let Some(format) = overrides.format.as_ref() {
            Self::try_new(format.as_ref())
        } else {
            Ok(self)
        }
    }
}

impl ArchivedDateSpec {
    /// Validates a date string against the format constraint directly from the
    /// database without deserialization.
    ///
    /// This is a zero-copy validation method that operates on the archived
    /// representation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self, value: &str) -> Result<(), SchemaError> {
        let is_valid =
            chrono::NaiveDateTime::parse_from_str(value, self.format.as_ref())
                .is_ok()
                || chrono::NaiveDate::parse_from_str(
                    value,
                    self.format.as_ref(),
                )
                .is_ok();

        if !is_valid {
            return Err(SchemaError::InvalidDateFormat(format!(
                "Value {value} does not match format {}",
                self.format.as_ref()
            )));
        }
        Ok(())
    }
}

impl TryFrom<crate::schema::raw::RawDateSpec> for DateSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        raw: crate::schema::raw::RawDateSpec,
    ) -> Result<Self, Self::Error> {
        let format = raw.format.ok_or_else(|| {
            SchemaError::ValidationFailed("date format is required".into())
        })?;
        Self::try_new(&format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_spec_accepts_valid_date() {
        let spec = DateSpec::try_new("%Y-%m-%dT%H:%M:%SZ")
            .expect("Expected valid DateSpec");
        let result = spec.validate_str("2024-01-15T14:30:00Z");
        assert!(
            result.is_ok(),
            "Expected date validation to succeed, got: {result:?}"
        );
    }

    #[test]
    fn date_spec_rejects_invalid_date() {
        let spec = DateSpec::try_new("%Y-%m-%dT%H:%M:%SZ")
            .expect("Expected valid DateSpec");
        let result = spec.validate_str("not-a-date");
        assert!(
            matches!(result, Err(SchemaError::InvalidDateFormat(_))),
            "Expected InvalidDateFormat error for invalid date string, got: \
             {result:?}"
        );
    }
}
