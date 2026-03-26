use std::borrow::Cow;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};

use crate::config::value::DateSpec;

/// Typed value extracted from inline field during parsing.
///
/// This enum supports heuristic type detection during ingestion, allowing
/// the parser to type field values once instead of forcing every consumer
/// to re-parse strings.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RawFieldValue<'source> {
    /// String value (fallback for unrecognized types).
    String(Cow<'source, str>),
    /// Numeric value (float).
    Number(f64),
    /// Date value (YYYY-MM-DD).
    Date(NaiveDate),
    /// Date/time value with offset.
    DateTime(DateTime<FixedOffset>),
    /// Wall clock time.
    Time(NaiveTime),
    /// Boolean value.
    Boolean(bool),
}

impl<'source> RawFieldValue<'source> {
    /// Attempt to parse a string into a typed value.
    ///
    /// Uses `DateSpec` format if provided for spec-aware parsing, otherwise
    /// falls back to heuristic parsing for common formats.
    ///
    /// # Type Detection Order
    /// 1. If `spec` provided: Try spec format first
    /// 2. Heuristic parsing:
    ///    - RFC3339 datetime
    ///    - Common date formats (YYYY-MM-DD, YYYY/MM/DD, etc.)
    ///    - Boolean (true/false, yes/no)
    ///    - Number (f64)
    /// 3. Fallback: String
    pub fn from_str_with_spec(
        text: &'source str,
        _key: &str,
        spec: Option<&DateSpec>,
    ) -> Self {
        // 1. Try spec format if provided
        if let Some(date_spec) = spec {
            if let Ok(d) = NaiveDate::parse_from_str(text, date_spec.format()) {
                return Self::Date(d);
            }
            if let Ok(dt) = DateTime::parse_from_str(text, date_spec.format()) {
                return Self::DateTime(dt);
            }
        }

        // 2. Heuristic parsing

        // Try RFC3339 datetime
        if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
            return Self::DateTime(dt);
        }

        // Try common date formats
        let date_formats =
            ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d", "%d-%m-%Y", "%d/%m/%Y"];
        for fmt in date_formats {
            if let Ok(d) = NaiveDate::parse_from_str(text, fmt) {
                return Self::Date(d);
            }
        }

        // Try time formats
        let time_formats = ["%H:%M:%S", "%H:%M"];
        for fmt in time_formats {
            if let Ok(t) = NaiveTime::parse_from_str(text, fmt) {
                return Self::Time(t);
            }
        }

        // Try boolean
        match text.trim().to_lowercase().as_str() {
            "true" | "yes" => return Self::Boolean(true),
            "false" | "no" => return Self::Boolean(false),
            _ => {}
        }

        // Try number
        if let Ok(n) = text.trim().parse::<f64>() {
            return Self::Number(n);
        }

        // 3. Fallback to string
        Self::String(Cow::Borrowed(text))
    }

    /// Convert to owned variant for crossing lifetime boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawFieldValue<'static> {
        match self {
            Self::String(s) => {
                RawFieldValue::String(Cow::Owned(s.into_owned()))
            }
            Self::Number(n) => RawFieldValue::Number(n),
            Self::Date(d) => RawFieldValue::Date(d),
            Self::DateTime(dt) => RawFieldValue::DateTime(dt),
            Self::Time(t) => RawFieldValue::Time(t),
            Self::Boolean(b) => RawFieldValue::Boolean(b),
        }
    }
}
