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

use chrono::{DateTime, TimeZone as _, Utc};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap as _,
};

use super::error::FieldValueError;

/// Shared primitive for dynamic note values (frontmatter and task metadata).
///
/// This enum represents the set of supported value types in the note domain.
/// It is used for both frontmatter (YAML/TOML) and task metadata (`[key::
/// value]`).
///
/// Note: `DateTime` is stored as an `i64` Unix timestamp for `rkyv`
/// compatibility.
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
    /// Date/time value (stored as Unix timestamp for serialization).
    Date(i64),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(#[rkyv(omit_bounds)] HashMap<Box<str>, FieldValue>),
    /// String value.
    String(Box<str>),
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
            Self::Number(_) => "number",
            Self::Object(_) => "object",
            Self::String(_) => "string",
        }
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

    /// Returns the date timestamp if this is a `Date` variant.
    #[inline]
    #[must_use]
    pub fn as_date(&self) -> Option<i64> {
        if let &Self::Date(timestamp) = self {
            Some(timestamp)
        } else {
            None
        }
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
            Some(obj)
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
            &Self::Date(ts) => {
                let datetime = DateTime::from_timestamp(ts, 0)
                    .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
                serializer.serialize_str(&datetime.to_rfc3339())
            }
            &Self::Number(n) => serializer.serialize_f64(n),
            #[expect(clippy::iter_over_hash_type, reason = "Internal matching")]
            Self::Object(obj) => {
                let mut map = serializer.serialize_map(Some(obj.len()))?;
                for (key, val) in obj {
                    map.serialize_entry(key, val)?;
                }
                map.end()
            }
            Self::String(s) => serializer.serialize_str(s),
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
                // Try parsing as ISO8601 date or YYYY-MM-DD
                if let Ok(dt) = DateTime::parse_from_rfc3339(v) {
                    return Ok(FieldValue::Date(dt.timestamp()));
                }
                if let Ok(dt) = chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d")
                    && let Some(naive) = dt.and_hms_opt(0, 0, 0)
                {
                    return Ok(FieldValue::Date(
                        Utc.from_utc_datetime(&naive).timestamp(),
                    ));
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
                Ok(FieldValue::Object(fields))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(FieldValue::String("".into()))
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
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError>;
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
    ) -> Result<Self, FieldValueError>;
}

// ----------------------------------------------------------- //
//                    Trait Implementations                    //
// ----------------------------------------------------------- //

impl TryFromFieldValue for bool {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError> {
        value.as_bool().ok_or_else(|| FieldValueError::TypeMismatch {
            expected: "boolean",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for f64 {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError> {
        value.as_number().ok_or_else(|| FieldValueError::TypeMismatch {
            expected: "number",
            actual: value.type_name(),
        })
    }
}

impl TryFromFieldValue for Box<str> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError> {
        value.as_str().map(Into::into).ok_or_else(|| {
            FieldValueError::TypeMismatch {
                expected: "string",
                actual: value.type_name(),
            }
        })
    }
}

impl TryFromFieldValue for DateTime<Utc> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError> {
        let ts =
            value.as_date().ok_or_else(|| FieldValueError::TypeMismatch {
                expected: "date",
                actual: value.type_name(),
            })?;
        Utc.timestamp_opt(ts, 0).single().ok_or({
            FieldValueError::InvalidDateTimestamp {
                timestamp: ts,
            }
        })
    }
}

impl TryFromFieldValue for Vec<Box<str>> {
    #[inline]
    fn try_from_value(value: &FieldValue) -> Result<Self, FieldValueError> {
        if let Some(arr) = value.as_array() {
            let mut out = Vec::with_capacity(arr.len());
            for (index, item) in arr.iter().enumerate() {
                let Some(s) = item.as_str() else {
                    return Err(FieldValueError::ArrayElementTypeMismatch {
                        index,
                        expected: "string",
                        actual: item.type_name(),
                    });
                };
                out.push(s.into());
            }
            return Ok(out);
        }

        value.as_str().map(|s| vec![s.into()]).ok_or_else(|| {
            FieldValueError::TypeMismatch {
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
    ) -> Result<Self, FieldValueError> {
        value.as_str().ok_or_else(|| FieldValueError::TypeMismatch {
            expected: "string",
            actual: value.type_name(),
        })
    }
}

impl<'value> TryFromFieldValueRef<'value> for &'value [FieldValue] {
    #[inline]
    fn try_from_value_ref(
        value: &'value FieldValue,
    ) -> Result<Self, FieldValueError> {
        value.as_array().ok_or_else(|| FieldValueError::TypeMismatch {
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
                FieldValue::Object(obj),
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
        assert_eq!(FieldValue::Date(0i64).type_name(), "date");
        assert_eq!(
            FieldValue::Array(vec![].into_boxed_slice()).type_name(),
            "array"
        );
        assert_eq!(FieldValue::Object(HashMap::new()).type_name(), "object");
    }
}
