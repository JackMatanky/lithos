//! Shared value primitive for note metadata (frontmatter and task metadata).
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::missing_trait_methods,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs, Error trait requires default impls that we don't use"
)]

use std::collections::HashMap;

use chrono::{
    DateTime, Datelike as _, FixedOffset, NaiveDate, NaiveTime, TimeDelta,
    TimeZone as _, Utc,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap as _,
};

use super::error::FrontmatterError;

/// Shared primitive for dynamic note values (frontmatter and task metadata).
///
/// This enum represents the set of supported value types in the note domain.
/// It is used for both frontmatter (YAML/TOML) and task metadata (`[key::
/// value]`).
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
)))]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum FieldValue {
    /// Array of values (stored as Box for memory efficiency).
    Array(#[rkyv(omit_bounds)] Box<[FieldValue]>),
    /// Boolean value.
    Boolean(bool),
    /// Date value (YYYY-MM-DD).
    Date(NaiveDateValue),
    /// Date/time value with offset.
    DateTime(DateTimeValue),
    /// Wall clock time.
    Time(NaiveTimeValue),
    /// Time duration.
    Duration(DurationValue),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    ///
    /// Boxed to keep the enum size small (improves cache performance for
    /// smaller variants).
    Object(#[rkyv(omit_bounds)] Box<HashMap<Box<str>, FieldValue>>),
    /// String value.
    String(Box<str>),
    /// Null/empty value.
    Null,
}

/// Internal archivable representation of a `chrono::NaiveDate`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NaiveDateValue {
    /// Days since January 1, 1 CE.
    num_days: i32,
}

impl From<NaiveDate> for NaiveDateValue {
    #[inline]
    fn from(date: NaiveDate) -> Self {
        Self {
            num_days: date.num_days_from_ce(),
        }
    }
}

impl From<NaiveDateValue> for NaiveDate {
    #[inline]
    fn from(val: NaiveDateValue) -> Self {
        NaiveDate::from_num_days_from_ce_opt(val.num_days).unwrap_or_default()
    }
}

/// Internal archivable representation of a `chrono::DateTime<FixedOffset>`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct DateTimeValue {
    /// Seconds since Unix epoch.
    timestamp: i64,
    /// Nanoseconds part.
    nanos: u32,
    /// Offset from UTC in seconds.
    offset_secs: i32,
}

impl From<DateTime<FixedOffset>> for DateTimeValue {
    #[inline]
    fn from(dt: DateTime<FixedOffset>) -> Self {
        Self {
            timestamp: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos(),
            offset_secs: dt.offset().local_minus_utc(),
        }
    }
}

impl From<DateTimeValue> for DateTime<FixedOffset> {
    #[inline]
    fn from(val: DateTimeValue) -> Self {
        #[expect(
            clippy::unwrap_used,
            reason = "Default offsets and epoch timestamps are guaranteed \
                      valid"
        )]
        let offset = FixedOffset::east_opt(val.offset_secs)
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());

        #[expect(
            clippy::unwrap_used,
            reason = "Epoch timestamp is guaranteed valid"
        )]
        offset.timestamp_opt(val.timestamp, val.nanos).single().unwrap_or_else(
            || Utc.timestamp_opt(0, 0).single().unwrap().with_timezone(&offset),
        )
    }
}

/// Internal archivable representation of a `chrono::NaiveTime`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NaiveTimeValue {
    /// Seconds since midnight.
    secs: u32,
    /// Nanoseconds part.
    nanos: u32,
}

impl From<NaiveTime> for NaiveTimeValue {
    #[inline]
    fn from(time: NaiveTime) -> Self {
        use chrono::Timelike as _;
        Self {
            secs: time.num_seconds_from_midnight(),
            nanos: time.nanosecond(),
        }
    }
}

impl From<NaiveTimeValue> for NaiveTime {
    #[inline]
    fn from(val: NaiveTimeValue) -> Self {
        NaiveTime::from_num_seconds_from_midnight_opt(val.secs, val.nanos)
            .unwrap_or_default()
    }
}

/// Internal archivable representation of a `chrono::TimeDelta`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct DurationValue {
    /// Total seconds.
    secs: i64,
    /// Nanoseconds part.
    nanos: u32,
}

impl From<TimeDelta> for DurationValue {
    #[inline]
    fn from(delta: TimeDelta) -> Self {
        #[expect(
            clippy::as_conversions,
            clippy::cast_sign_loss,
            reason = "Sign is handled manually"
        )]
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "Manual math for negative durations"
        )]
        let (secs, nanos) = if delta.subsec_nanos() < 0i32 {
            // Adjust negative subsecond nanoseconds to positive by borrowing a
            // second. e.g. -0.5s -> num_seconds=0,
            // subsec_nanos=-500ms Result: secs=-1, nanos=500ms
            // delta = -1s + 0.5s = -0.5s (correct)
            (
                delta.num_seconds() - 1i64,
                (1_000_000_000i32 + delta.subsec_nanos()) as u32,
            )
        } else {
            (delta.num_seconds(), delta.subsec_nanos() as u32)
        };

        Self {
            secs,
            nanos,
        }
    }
}

impl From<DurationValue> for TimeDelta {
    #[inline]
    fn from(val: DurationValue) -> Self {
        TimeDelta::new(val.secs, val.nanos).unwrap_or_else(TimeDelta::zero)
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Accessor methods intentionally use match ergonomics on `&self` \
              (e.g., `if let Self::Array(arr) = self`) to avoid `ref` \
              patterns and keep the code concise"
)]
impl FieldValue {
    /// Returns the human-readable type name.
    #[inline]
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match *self {
            Self::Array(_) => "array",
            Self::Boolean(_) => "boolean",
            Self::Date(_) => "date",
            Self::DateTime(_) => "datetime",
            Self::Time(_) => "time",
            Self::Duration(_) => "duration",
            Self::Number(_) => "number",
            Self::Object(_) => "object",
            Self::String(_) => "string",
            Self::Null => "null",
        }
    }

    /// Returns true if this is a `Null` variant.
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the boolean value if this is a `Boolean` variant.
    #[inline]
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let &Self::Boolean(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns the date value if this is a `Date` variant.
    #[inline]
    #[must_use]
    pub fn as_naive_date(&self) -> Option<NaiveDate> {
        if let &Self::Date(val) = self {
            Some(val.into())
        } else {
            None
        }
    }

    /// Returns the datetime value if this is a `DateTime` variant.
    #[inline]
    #[must_use]
    pub fn as_datetime(&self) -> Option<DateTime<FixedOffset>> {
        if let &Self::DateTime(val) = self {
            Some(val.into())
        } else {
            None
        }
    }

    /// Returns the time value if this is a `Time` variant.
    #[inline]
    #[must_use]
    pub fn as_naive_time(&self) -> Option<NaiveTime> {
        if let &Self::Time(val) = self {
            Some(val.into())
        } else {
            None
        }
    }

    /// Returns the duration value if this is a `Duration` variant.
    #[inline]
    #[must_use]
    pub fn as_duration(&self) -> Option<TimeDelta> {
        if let &Self::Duration(val) = self {
            Some(val.into())
        } else {
            None
        }
    }

    /// Returns true if this value represents a date or time.
    #[inline]
    #[must_use]
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::Date(_) | Self::DateTime(_) | Self::Time(_))
    }

    /// Returns the number value if this is a `Number` variant.
    #[inline]
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        if let &Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    /// Returns the string value if this is a `String` variant.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the array slice if this is an `Array` variant.
    #[inline]
    #[must_use]
    pub fn as_array(&self) -> Option<&[FieldValue]> {
        if let Self::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    /// Returns the object map if this is an `Object` variant.
    #[inline]
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<Box<str>, FieldValue>> {
        if let Self::Object(obj) = self {
            Some(obj.as_ref())
        } else {
            None
        }
    }

    /// Returns an iterator over array items if this is an `Array` variant.
    #[inline]
    #[must_use]
    pub fn array_items(&self) -> Option<FieldArrayItems<'_>> {
        self.as_array().map(|arr| FieldArrayItems {
            inner: arr.iter(),
        })
    }

    /// Returns an iterator over object fields if this is an `Object` variant.
    #[inline]
    #[must_use]
    pub fn object_fields(&self) -> Option<FieldObjectFields<'_>> {
        self.as_object().map(|obj| FieldObjectFields {
            inner: obj.iter(),
        })
    }

    /// Convert this `FieldValue` to a JSON string for indexing.
    ///
    /// This provides a stable string representation for metadata indexes.
    /// Uses `serde_json` for robust escaping and consistent formatting.
    #[inline]
    #[must_use]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Attempts to parse a string variant as a [`NaiveDate`] with a custom
    /// format.
    #[inline]
    #[must_use]
    pub fn parse_as_date(&self, format: &str) -> Option<NaiveDate> {
        let s = self.as_str()?;
        NaiveDate::parse_from_str(s, format).ok()
    }

    /// Attempts to parse a string variant as a [`DateTime<FixedOffset>`] with a
    /// custom format.
    #[inline]
    #[must_use]
    pub fn parse_as_datetime(
        &self,
        format: &str,
    ) -> Option<DateTime<FixedOffset>> {
        let s = self.as_str()?;
        DateTime::parse_from_str(s, format).ok()
    }

    /// Attempts to parse a string variant as a [`NaiveTime`] with a custom
    /// format.
    #[inline]
    #[must_use]
    pub fn parse_as_time(&self, format: &str) -> Option<NaiveTime> {
        let s = self.as_str()?;
        NaiveTime::parse_from_str(s, format).ok()
    }
}

impl Serialize for FieldValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Intentional use of match ergonomics on &self"
        )]
        match self {
            Self::Array(arr) => serializer.collect_seq(arr.iter()),
            &Self::Boolean(b) => serializer.serialize_bool(b),
            &Self::Date(val) => {
                let dt: NaiveDate = val.into();
                serializer.serialize_str(&dt.to_string())
            }
            &Self::DateTime(val) => {
                let dt: DateTime<FixedOffset> = val.into();
                serializer.serialize_str(&dt.to_rfc3339())
            }
            &Self::Time(val) => {
                let t: NaiveTime = val.into();
                serializer.serialize_str(&t.to_string())
            }
            &Self::Duration(val) => {
                let d: TimeDelta = val.into();
                serializer.serialize_str(&d.to_string())
            }
            Self::Number(n) => serializer.serialize_f64(*n),
            #[expect(clippy::iter_over_hash_type, reason = "Internal matching")]
            Self::Object(obj) => {
                let mut map = serializer.serialize_map(Some(obj.len()))?;
                for (key, val) in obj.as_ref() {
                    map.serialize_entry(key, val)?;
                }
                map.end()
            }
            Self::String(s) => serializer.serialize_str(s),
            Self::Null => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for FieldValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, SeqAccess, Visitor};

        struct FieldValueVisitor;

        impl<'de> Visitor<'de> for FieldValueVisitor {
            type Value = FieldValue;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                formatter.write_str("any valid metadata value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(FieldValue::Boolean(v))
            }

            #[expect(
                clippy::as_conversions,
                clippy::cast_precision_loss,
                reason = "Domain values fit f64"
            )]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(FieldValue::Number(v as f64))
            }

            #[expect(
                clippy::as_conversions,
                clippy::cast_precision_loss,
                reason = "Domain values fit f64"
            )]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(FieldValue::Number(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(FieldValue::Number(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // Try parsing prioritized temporal types
                if let Ok(dt) = DateTime::parse_from_rfc3339(v) {
                    return Ok(FieldValue::DateTime(dt.into()));
                }
                if let Ok(d) = NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                    return Ok(FieldValue::Date(d.into()));
                }
                if let Ok(t) = NaiveTime::parse_from_str(v, "%H:%M:%S") {
                    return Ok(FieldValue::Time(t.into()));
                }
                if let Ok(t) = NaiveTime::parse_from_str(v, "%H:%M") {
                    return Ok(FieldValue::Time(t.into()));
                }
                Ok(FieldValue::String(v.into()))
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
                Ok(FieldValue::Array(values.into_boxed_slice()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut fields =
                    HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, value)) =
                    map.next_entry::<Box<str>, FieldValue>()?
                {
                    fields.insert(key, value);
                }
                Ok(FieldValue::Object(Box::new(fields)))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(FieldValue::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(FieldValue::Null)
            }
        }

        deserializer.deserialize_any(FieldValueVisitor)
    }
}

/// Borrowed iterator over object fields in a [`FieldValue::Object`].
pub struct FieldObjectFields<'value> {
    inner: std::collections::hash_map::Iter<'value, Box<str>, FieldValue>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'value> Iterator for FieldObjectFields<'value> {
    type Item = (&'value str, &'value FieldValue);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, value)| (key.as_ref(), value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Borrowed iterator over array items in a [`FieldValue::Array`].
pub struct FieldArrayItems<'value> {
    inner: std::slice::Iter<'value, FieldValue>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'value> Iterator for FieldArrayItems<'value> {
    type Item = &'value FieldValue;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// ----------------------------------------------------------- //
//                      Trait Definitions                      //
// ----------------------------------------------------------- //

/// Fallible, strict conversions from [`FieldValue`].
///
/// This is intentionally a *local* trait (instead of `TryFrom<&FieldValue>`) to
/// avoid Rust's orphan rules (we can't implement foreign traits for foreign
/// types like `bool`, `f64`, `String`, etc.).
pub trait TryFromFieldValue: Sized {
    /// Attempts to extract a value of type `Self` from a [`FieldValue`].
    ///
    /// Returns a structured error when the value is present but incompatible.
    ///
    /// # Errors
    ///
    /// Returns a structured error describing why the conversion failed.
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError>;
}

/// Fallible, strict conversions from a borrowed [`FieldValue`].
///
/// This exists to support *non-owning* access patterns like `&str` and slices.
pub trait TryFromFieldValueRef<'value>: Sized {
    /// Attempts to extract a value of type `Self` from a borrowed
    /// [`FieldValue`].
    ///
    /// # Errors
    ///
    /// Returns a structured error describing why the conversion failed.
    fn try_from_value_ref(
        value: &'value FieldValue,
    ) -> Result<Self, FrontmatterError>;
}

// ----------------------------------------------------------- //
//                    Trait Implementations                    //
// ----------------------------------------------------------- //

impl TryFromFieldValue for bool {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_bool().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "boolean",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for f64 {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_number().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "number",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for Box<str> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_str().map(Into::into).ok_or_else(|| {
            FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: "string",
                actual: value.type_name(),
            }
        })
    }
}

impl TryFromFieldValue for DateTime<Utc> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(dt) = value.as_datetime() {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Some(d) = value.as_naive_date()
            && let Some(naive) = d.and_hms_opt(0, 0, 0)
        {
            return Ok(Utc.from_utc_datetime(&naive));
        }
        Err(FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "datetime",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for DateTime<FixedOffset> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(dt) = value.as_datetime() {
            return Ok(dt);
        }
        if let Some(d) = value.as_naive_date() {
            #[expect(clippy::unwrap_used, reason = "Zero offset is valid")]
            let offset = FixedOffset::east_opt(0).unwrap();
            if let Some(naive) = d.and_hms_opt(0, 0, 0) {
                return Ok(offset.from_utc_datetime(&naive));
            }
        }
        Err(FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "datetime",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for NaiveDate {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(d) = value.as_naive_date() {
            return Ok(d);
        }
        if let Some(dt) = value.as_datetime() {
            return Ok(dt.date_naive());
        }
        Err(FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "date",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for NaiveTime {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(t) = value.as_naive_time() {
            return Ok(t);
        }
        if let Some(dt) = value.as_datetime() {
            return Ok(dt.time());
        }
        Err(FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "time",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for TimeDelta {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        value.as_duration().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "duration",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for Vec<Box<str>> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FrontmatterError> {
        if let Some(arr) = value.as_array() {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                let Some(s) = item.as_str() else {
                    return Err(FrontmatterError::TypeMismatch {
                        key: "".into(),
                        expected: "string",
                        actual: item.type_name(),
                    });
                };
                out.push(s.into());
            }
            return Ok(out);
        }

        value.as_str().map(|s| vec![s.into()]).ok_or_else(|| {
            FrontmatterError::TypeMismatch {
                key: "".into(),
                expected: "array",
                actual: value.type_name(),
            }
        })
    }
}

impl<'value> TryFromFieldValueRef<'value> for &'value str {
    #[inline]
    fn try_from_value_ref(
        value: &'value FieldValue,
    ) -> Result<Self, FrontmatterError> {
        value.as_str().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "string",
            actual: value.type_name(),
        })
    }
}

impl<'value> TryFromFieldValueRef<'value> for &'value [FieldValue] {
    #[inline]
    fn try_from_value_ref(
        value: &'value FieldValue,
    ) -> Result<Self, FrontmatterError> {
        value.as_array().ok_or_else(|| FrontmatterError::TypeMismatch {
            key: "".into(),
            expected: "array",
            actual: value.type_name(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    mod fixtures {
        use std::collections::HashMap;

        use super::*;

        pub fn complex_value() -> FieldValue {
            let mut obj = HashMap::new();
            obj.insert("key".into(), FieldValue::String("val".into()));
            FieldValue::Array(
                vec![
                    FieldValue::Number(1.0),
                    FieldValue::Boolean(true),
                    FieldValue::Object(Box::new(obj)),
                    FieldValue::Null,
                ]
                .into_boxed_slice(),
            )
        }
    }

    mod accessors {
        use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta};
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::string(FieldValue::String("".into()), "string")]
        #[case::number(FieldValue::Number(0.0), "number")]
        #[case::boolean(FieldValue::Boolean(true), "boolean")]
        #[case::date(
            FieldValue::Date(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().into()
            ),
            "date"
        )]
        #[case::datetime(
            FieldValue::DateTime(
                DateTime::parse_from_rfc3339("2024-03-20T14:30:00Z")
                    .unwrap()
                    .into()
            ),
            "datetime"
        )]
        #[case::time(
            FieldValue::Time(
                NaiveTime::from_hms_opt(14, 30, 0).unwrap().into()
            ),
            "time"
        )]
        #[case::duration(
            FieldValue::Duration(TimeDelta::hours(2).into()),
            "duration"
        )]
        #[case::array(FieldValue::Array(vec![].into_boxed_slice()), "array")]
        #[case::object(FieldValue::Object(Box::default()), "object")]
        #[case::null(FieldValue::Null, "null")]
        fn type_name_should_reflect_variant(
            #[case] val: FieldValue,
            #[case] expected: &str,
        ) {
            assert_eq!(
                val.type_name(),
                expected,
                "Type name mismatch for {expected}",
            );
        }

        #[test]
        fn is_temporal_should_detect_date_time_variants() {
            assert!(
                FieldValue::Date(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().into()
                )
                .is_temporal(),
                "Date should be temporal"
            );
            assert!(
                FieldValue::DateTime(
                    DateTime::parse_from_rfc3339("2024-03-20T14:30:00Z")
                        .unwrap()
                        .into()
                )
                .is_temporal(),
                "DateTime should be temporal"
            );
            assert!(
                FieldValue::Time(
                    NaiveTime::from_hms_opt(14, 30, 0).unwrap().into()
                )
                .is_temporal(),
                "Time should be temporal"
            );
            assert!(
                !FieldValue::String("".into()).is_temporal(),
                "String should NOT be temporal"
            );
        }

        #[test]
        fn is_null_should_return_true_for_null_variant() {
            assert!(
                FieldValue::Null.is_null(),
                "Null.is_null() should be true"
            );
            assert!(
                !FieldValue::Boolean(false).is_null(),
                "Boolean.is_null() should be false"
            );
        }

        #[test]
        fn as_methods_should_return_some_for_correct_variant() {
            assert_eq!(FieldValue::Boolean(true).as_bool(), Some(true));
            assert_eq!(
                FieldValue::Number(1.0).as_number(),
                Some(1.0f64),
                "Number mismatch"
            );
            assert_eq!(FieldValue::String("hi".into()).as_str(), Some("hi"));

            let date = NaiveDate::from_ymd_opt(2024, 3, 22).unwrap();
            assert_eq!(
                FieldValue::Date(date.into()).as_naive_date(),
                Some(date)
            );

            let dt =
                DateTime::parse_from_rfc3339("2024-03-22T14:30:00Z").unwrap();
            assert_eq!(FieldValue::DateTime(dt.into()).as_datetime(), Some(dt));

            let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
            assert_eq!(
                FieldValue::Time(time.into()).as_naive_time(),
                Some(time)
            );

            let dur = TimeDelta::seconds(10);
            assert_eq!(
                FieldValue::Duration(dur.into()).as_duration(),
                Some(dur)
            );
        }

        #[test]
        fn collection_accessors_should_return_correct_types() {
            let val = fixtures::complex_value();
            assert!(val.as_array().is_some(), "Array accessor failed");
            assert!(val.array_items().is_some(), "Array items iterator failed");

            let obj_val = val
                .as_array()
                .and_then(|a| a.get(2))
                .cloned()
                .expect("Object variant expected at index 2");
            assert!(obj_val.as_object().is_some(), "Object accessor failed");
            assert!(
                obj_val.object_fields().is_some(),
                "Object fields iterator failed"
            );
        }

        #[test]
        fn as_methods_should_return_none_for_mismatched_variants() {
            let val = FieldValue::Null;
            assert!(val.as_bool().is_none());
            assert!(val.as_number().is_none());
            assert!(val.as_str().is_none());
            assert!(val.as_naive_date().is_none());
            assert!(val.as_datetime().is_none());
            assert!(val.as_naive_time().is_none());
            assert!(val.as_duration().is_none());
            assert!(val.as_array().is_none());
            assert!(val.as_object().is_none());
        }
    }

    mod conversions {

        use super::*;

        mod owned {
            use super::*;

            #[test]
            fn should_convert_to_bool() {
                let val = FieldValue::Boolean(true);
                assert!(
                    bool::try_from_value(&val).unwrap(),
                    "Failed to convert Boolean to bool"
                );
            }

            #[test]
            fn should_convert_to_number() {
                let val = FieldValue::Number(42.5);
                #[expect(clippy::float_cmp, reason = "Exact value expected")]
                {
                    assert_eq!(
                        f64::try_from_value(&val).unwrap(),
                        42.5f64,
                        "Failed to convert Number to f64"
                    );
                }
            }

            #[test]
            fn should_convert_to_box_str() {
                let val = FieldValue::String("hello".into());
                assert_eq!(
                    Box::<str>::try_from_value(&val).unwrap().as_ref(),
                    "hello",
                    "Failed to convert String to Box<str>"
                );
            }

            #[test]
            fn should_convert_date_to_datetime_utc() {
                let date = NaiveDate::from_ymd_opt(2024, 3, 22).unwrap();
                let val = FieldValue::Date(date.into());
                let expected =
                    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
                assert_eq!(
                    DateTime::<Utc>::try_from_value(&val).unwrap(),
                    expected,
                    "Failed to convert Date to DateTime<Utc>"
                );
            }

            #[test]
            fn should_convert_temporal_variants_to_native_types() {
                let date = NaiveDate::from_ymd_opt(2024, 3, 22).unwrap();
                let val_date = FieldValue::Date(date.into());
                assert_eq!(NaiveDate::try_from_value(&val_date).unwrap(), date);

                let dt = DateTime::parse_from_rfc3339("2024-03-22T14:30:00Z")
                    .unwrap();
                let val_datetime = FieldValue::DateTime(dt.into());
                assert_eq!(
                    DateTime::<FixedOffset>::try_from_value(&val_datetime)
                        .unwrap(),
                    dt
                );
                // Crossing types: DateTime to NaiveDate
                assert_eq!(
                    NaiveDate::try_from_value(&val_datetime).unwrap(),
                    dt.date_naive()
                );
                // Crossing types: DateTime to NaiveTime
                assert_eq!(
                    NaiveTime::try_from_value(&val_datetime).unwrap(),
                    dt.time()
                );

                let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
                let val_time = FieldValue::Time(time.into());
                assert_eq!(NaiveTime::try_from_value(&val_time).unwrap(), time);

                let dur = TimeDelta::seconds(10);
                let val_dur = FieldValue::Duration(dur.into());
                assert_eq!(TimeDelta::try_from_value(&val_dur).unwrap(), dur);
            }

            #[test]
            fn should_convert_string_or_array_to_vec_box_str() {
                // Single string case
                let val_s = FieldValue::String("tag1".into());
                let res_s = Vec::<Box<str>>::try_from_value(&val_s).unwrap();
                assert_eq!(
                    res_s,
                    vec!["tag1".into()],
                    "Failed to convert single String to Vec<Box<str>>"
                );

                // Array case
                let val_a = FieldValue::Array(
                    vec![
                        FieldValue::String("t1".into()),
                        FieldValue::String("t2".into()),
                    ]
                    .into_boxed_slice(),
                );
                let res_a = Vec::<Box<str>>::try_from_value(&val_a).unwrap();
                assert_eq!(
                    res_a,
                    vec!["t1".into(), "t2".into()],
                    "Failed to convert Array of Strings to Vec<Box<str>>"
                );
            }

            #[test]
            fn should_fail_on_type_mismatch() {
                let val = FieldValue::Null;
                let res = bool::try_from_value(&val);
                assert!(
                    matches!(res, Err(FrontmatterError::TypeMismatch { .. })),
                    "Expected TypeMismatch error, got {res:?}"
                );
            }
        }

        mod borrowed {
            use super::*;

            #[test]
            fn should_convert_to_str_slice() {
                let val = FieldValue::String("borrowed".into());
                assert_eq!(
                    <&str>::try_from_value_ref(&val).unwrap(),
                    "borrowed",
                    "Failed to borrow &str from String variant"
                );
            }

            #[test]
            fn should_convert_to_field_slice() {
                let val = FieldValue::Array(
                    vec![FieldValue::Boolean(true), FieldValue::Number(1.0)]
                        .into_boxed_slice(),
                );
                let slice = <&[FieldValue]>::try_from_value_ref(&val).unwrap();
                assert_eq!(slice.len(), 2, "Borrowed slice length mismatch");
                assert_eq!(
                    slice.first().and_then(FieldValue::as_bool),
                    Some(true),
                    "Borrowed slice content mismatch"
                );
            }

            #[test]
            fn should_fail_on_borrow_type_mismatch() {
                let val = FieldValue::Boolean(true);
                let res = <&str>::try_from_value_ref(&val);
                assert!(
                    matches!(res, Err(FrontmatterError::TypeMismatch { .. })),
                    "Expected TypeMismatch error when borrowing &str from \
                     bool, got {res:?}"
                );
            }
        }
        use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};

        use crate::note::error::FrontmatterError;
    }

    mod parsing {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::date("2024-03-22", "%Y-%m-%d")]
        #[case::datetime("2024-03-22T14:30:00+00:00", "%Y-%m-%dT%H:%M:%S%z")]
        #[case::time("14:30", "%H:%M")]
        fn should_manually_parse_temporal_strings(
            #[case] input: &str,
            #[case] format: &str,
        ) {
            let val = FieldValue::String(input.into());
            if format.contains('T') || format.contains('z') {
                assert!(
                    val.parse_as_datetime(format).is_some(),
                    "Failed to parse datetime: {input}"
                );
            } else if format.contains(':') {
                assert!(
                    val.parse_as_time(format).is_some(),
                    "Failed to parse time: {input}"
                );
            } else {
                assert!(
                    val.parse_as_date(format).is_some(),
                    "Failed to parse date: {input}"
                );
            }
        }
    }

    mod integrity {
        use super::*;

        mod serde_heuristics {
            use rstest::rstest;

            use super::*;

            #[rstest]
            #[case::date("\"2024-03-22\"", "date")]
            #[case::datetime("\"2024-03-22T14:30:00Z\"", "datetime")]
            #[case::time_full("\"14:30:00\"", "time")]
            #[case::time_short("\"14:30\"", "time")]
            #[case::plain_string("\"not-a-date\"", "string")]
            fn should_auto_detect_temporal_strings(
                #[case] json: &str,
                #[case] expected_type: &str,
            ) {
                let val: FieldValue = serde_json::from_str(json).unwrap();
                assert_eq!(
                    val.type_name(),
                    expected_type,
                    "Heuristic mismatch for {json}"
                );
            }
        }
        #[test]
        fn json_serialization_should_roundtrip() {
            let val = fixtures::complex_value();
            let json = val.to_json_string();
            let back: FieldValue = serde_json::from_str(&json).unwrap();
            assert_eq!(val, back, "JSON roundtrip failed");
        }

        #[test]
        fn rkyv_serialization_should_roundtrip() {
            let val = fixtures::complex_value();
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&val)
                .expect("rkyv serialize");
            let back: FieldValue =
                rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes)
                    .expect("rkyv deserialize");
            assert_eq!(val, back, "rkyv roundtrip failed");
        }
    }

    mod temporal {
        use chrono::{DateTime, FixedOffset, NaiveDate, TimeDelta};

        use super::*;

        #[test]
        fn naive_date_value_should_handle_extremes() {
            let d_min = NaiveDate::MIN;
            let val_min = NaiveDateValue::from(d_min);
            assert_eq!(
                NaiveDate::from(val_min),
                d_min,
                "Failed roundtrip for NaiveDate::MIN"
            );

            let d_max = NaiveDate::MAX;
            let val_max = NaiveDateValue::from(d_max);
            assert_eq!(
                NaiveDate::from(val_max),
                d_max,
                "Failed roundtrip for NaiveDate::MAX"
            );
        }

        #[test]
        fn date_time_value_should_handle_offset_extremes() {
            // chrono FixedOffset supports up to 24 hours
            let offset_east = FixedOffset::east_opt(86399).unwrap();
            let dt_east =
                offset_east.timestamp_opt(123_456_789, 0).single().unwrap();
            let val_east = DateTimeValue::from(dt_east);
            assert_eq!(
                DateTime::<FixedOffset>::from(val_east),
                dt_east,
                "Failed roundtrip for extreme positive offset"
            );

            let offset_west = FixedOffset::west_opt(86399).unwrap();
            let dt_west =
                offset_west.timestamp_opt(123_456_789, 0).single().unwrap();
            let val_west = DateTimeValue::from(dt_west);
            assert_eq!(
                DateTime::<FixedOffset>::from(val_west),
                dt_west,
                "Failed roundtrip for extreme negative offset"
            );
        }

        #[test]
        fn duration_value_should_handle_large_deltas() {
            let d = TimeDelta::weeks(1000);
            let val = DurationValue::from(d);
            assert_eq!(
                TimeDelta::from(val),
                d,
                "Failed roundtrip for large Duration"
            );
        }
    }

    mod proptests {
        use chrono::{NaiveDate, TimeDelta};
        use proptest::prelude::*;

        use super::*;
        proptest! {
            #[test]
            fn proptest_naive_date_roundtrip(days in -1_000_000i32..1_000_000i32) {
                if let Some(date) = NaiveDate::from_num_days_from_ce_opt(days) {
                    let val = NaiveDateValue::from(date);
                    prop_assert_eq!(NaiveDate::from(val), date);
                }
            }

            #[test]
            fn proptest_duration_roundtrip(secs in -1_000_000_000..1_000_000_000i64, nanos in 0..1_000_000_000u32) {
                if let Some(delta) = TimeDelta::new(secs, nanos) {
                    let val = DurationValue::from(delta);
                    prop_assert_eq!(TimeDelta::from(val), delta);
                }
            }
        }
    }
}
