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
            reason = "Nanoseconds are positive"
        )]
        Self {
            secs: delta.num_seconds(),
            nanos: delta.subsec_nanos() as u32,
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
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn serialization_round_trip() {
        let mut obj = HashMap::new();
        obj.insert("key".into(), FieldValue::String("val".into()));
        let val = FieldValue::Array(
            vec![
                FieldValue::Number(1.0),
                FieldValue::Boolean(true),
                FieldValue::Object(Box::new(obj)),
                FieldValue::Null,
            ]
            .into_boxed_slice(),
        );

        let json = serde_json::to_string(&val).unwrap();
        let back: FieldValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, back);
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn rkyv_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let val = FieldValue::Array(
            vec![
                FieldValue::String("test".into()),
                FieldValue::Number(123.0f64),
            ]
            .into_boxed_slice(),
        );

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&val)?;
        let deserialized: FieldValue =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes)?;

        assert_eq!(val, deserialized);
        Ok(())
    }

    #[test]
    fn type_name_detection() {
        assert_eq!(FieldValue::String("".into()).type_name(), "string");
        assert_eq!(FieldValue::Number(0.0f64).type_name(), "number");
        assert_eq!(FieldValue::Boolean(true).type_name(), "boolean");
        assert_eq!(
            FieldValue::Date(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().into()
            )
            .type_name(),
            "date"
        );
        assert_eq!(
            FieldValue::DateTime(
                DateTime::parse_from_rfc3339("2024-03-20T14:30:00Z")
                    .unwrap()
                    .into()
            )
            .type_name(),
            "datetime"
        );
        assert_eq!(
            FieldValue::Time(
                NaiveTime::from_hms_opt(14, 30, 0).unwrap().into()
            )
            .type_name(),
            "time"
        );
        assert_eq!(
            FieldValue::Duration(TimeDelta::hours(2).into()).type_name(),
            "duration"
        );
        assert_eq!(
            FieldValue::Array(vec![].into_boxed_slice()).type_name(),
            "array"
        );
        assert_eq!(FieldValue::Object(Box::default()).type_name(), "object");
        assert_eq!(FieldValue::Null.type_name(), "null");
    }

    #[test]
    fn temporal_parsing() {
        let json_date = "\"2024-03-20\"";
        let val_date: FieldValue = serde_json::from_str(json_date).unwrap();
        assert!(matches!(val_date, FieldValue::Date(_)));

        let json_dt = "\"2024-03-20T14:30:00Z\"";
        let val_dt: FieldValue = serde_json::from_str(json_dt).unwrap();
        assert!(matches!(val_dt, FieldValue::DateTime(_)));

        let json_time = "\"14:30:00\"";
        let val_time: FieldValue = serde_json::from_str(json_time).unwrap();
        assert!(matches!(val_time, FieldValue::Time(_)));

        let json_null = "null";
        let val_null: FieldValue = serde_json::from_str(json_null).unwrap();
        assert!(matches!(val_null, FieldValue::Null));
    }

    #[test]
    fn conversion_traits() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let val_date = FieldValue::Date(date.into());
        assert_eq!(val_date.as_naive_date(), Some(date));

        let dt =
            DateTime::parse_from_rfc3339("2024-03-20T14:30:00+01:00").unwrap();
        let val_dt = FieldValue::DateTime(dt.into());
        assert_eq!(val_dt.as_datetime(), Some(dt));

        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        let val_time = FieldValue::Time(time.into());
        assert_eq!(val_time.as_naive_time(), Some(time));

        let duration = TimeDelta::hours(2);
        let val_dur = FieldValue::Duration(duration.into());
        assert_eq!(val_dur.as_duration(), Some(duration));
    }

    #[test]
    fn manual_temporal_parsing() {
        let val_date = FieldValue::String("21-03-2024".into());
        let date = val_date.parse_as_date("%d-%m-%Y").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 21).unwrap());

        let val_dt = FieldValue::String("2024-03-21T14:30:00+00:00".into());
        let dt = val_dt.parse_as_datetime("%Y-%m-%dT%H:%M:%S%z").unwrap();
        assert_eq!(dt.timestamp(), 1_711_031_400);

        let val_time = FieldValue::String("14:30".into());
        let time = val_time.parse_as_time("%H:%M").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(14, 30, 0).unwrap());

        let val_invalid = FieldValue::String("not a date".into());
        assert!(val_invalid.parse_as_date("%Y-%m-%d").is_none());
    }
}
