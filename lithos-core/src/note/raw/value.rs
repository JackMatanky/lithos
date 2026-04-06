use std::{borrow::Cow, collections::HashMap};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::config::value::DateSpec;

/// Typed value extracted during lexical parsing.
///
/// This enum supports heuristic type detection during ingestion for inline
/// fields and frontmatter, allowing the parser to type field values once
/// instead of forcing every consumer to re-parse strings.
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
    /// Array of values.
    Array(Box<[RawFieldValue<'source>]>),
    /// Nested object values.
    Object(HashMap<Box<str>, RawFieldValue<'source>>),
    /// Null/empty value.
    Null,
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
            return Self::String(Cow::Borrowed(text));
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
        let trimmed = text.trim();
        if trimmed.eq_ignore_ascii_case("true")
            || trimmed.eq_ignore_ascii_case("yes")
        {
            return Self::Boolean(true);
        }
        if trimmed.eq_ignore_ascii_case("false")
            || trimmed.eq_ignore_ascii_case("no")
        {
            return Self::Boolean(false);
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
            Self::Array(values) => RawFieldValue::Array(
                values
                    .into_iter()
                    .map(RawFieldValue::into_owned)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            Self::Object(values) => RawFieldValue::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, value.into_owned()))
                    .collect(),
            ),
            Self::Null => RawFieldValue::Null,
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Serde defaults are sufficient for raw value deserialization"
)]
impl<'de> Deserialize<'de> for RawFieldValue<'static> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawFieldValueVisitor;

        #[expect(
            clippy::missing_trait_methods,
            reason = "Visitor only needs core value handlers"
        )]
        impl<'de> Visitor<'de> for RawFieldValueVisitor {
            type Value = RawFieldValue<'static>;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                formatter.write_str("any valid metadata value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Boolean(v))
            }

            #[expect(
                clippy::as_conversions,
                clippy::cast_precision_loss,
                reason = "Domain values fit f64"
            )]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Number(v as f64))
            }

            #[expect(
                clippy::as_conversions,
                clippy::cast_precision_loss,
                reason = "Domain values fit f64"
            )]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Number(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Number(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(dt) = DateTime::parse_from_rfc3339(v) {
                    return Ok(RawFieldValue::DateTime(dt));
                }
                if let Ok(d) = NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                    return Ok(RawFieldValue::Date(d));
                }
                if let Ok(t) = NaiveTime::parse_from_str(v, "%H:%M:%S") {
                    return Ok(RawFieldValue::Time(t));
                }
                if let Ok(t) = NaiveTime::parse_from_str(v, "%H:%M") {
                    return Ok(RawFieldValue::Time(t));
                }
                Ok(RawFieldValue::String(Cow::Owned(v.to_owned())))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values =
                    Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(RawFieldValue::Array(values.into_boxed_slice()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut fields =
                    HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, value)) =
                    map.next_entry::<Box<str>, RawFieldValue<'static>>()?
                {
                    fields.insert(key, value);
                }
                Ok(RawFieldValue::Object(fields))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(RawFieldValue::Null)
            }
        }

        deserializer.deserialize_any(RawFieldValueVisitor)
    }
}
