//! Shared value primitive for note metadata (frontmatter and task metadata).
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::collections::HashMap;

/// Shared primitive for dynamic note values (frontmatter and task metadata).
///
/// This enum represents the set of supported value types in the note domain.
/// It is used for both frontmatter (YAML/TOML) and task metadata (`[key::
/// value]`).
///
/// Note: `DateTime` stored as i64 timestamp for rkyv compatibility.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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
    /// Array of values.
    Array(#[rkyv(omit_bounds)] Vec<FieldValue>),
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

/// A high-level type descriptor for [`FieldValue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValueType {
    /// Array of field values.
    Array,
    /// Boolean value.
    Boolean,
    /// Date timestamp.
    Date,
    /// Floating point number.
    Number,
    /// Map of string keys to field values.
    Object,
    /// String value.
    String,
}

impl core::fmt::Display for FieldValueType {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match *self {
            Self::Array => "array",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::Number => "number",
            Self::Object => "object",
            Self::String => "string",
        };
        f.write_str(name)
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Accessor methods intentionally use match ergonomics on `&self` \
              (e.g., `if let Self::Array(arr) = self`) to avoid `ref` \
              patterns and keep the code concise"
)]
impl FieldValue {
    /// Returns the value type descriptor.
    #[inline]
    #[must_use]
    pub const fn value_type(&self) -> FieldValueType {
        match *self {
            Self::Array(_) => FieldValueType::Array,
            Self::Boolean(_) => FieldValueType::Boolean,
            Self::Date(_) => FieldValueType::Date,
            Self::Number(_) => FieldValueType::Number,
            Self::Object(_) => FieldValueType::Object,
            Self::String(_) => FieldValueType::String,
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

    /// Convert a `serde_json::Value` into a `FieldValue`.
    ///
    /// This is useful for adapters parsing JSON-like structures (including
    /// YAML/TOML converted via serde).
    #[inline]
    #[must_use]
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => {
                Self::String(s.clone().into_boxed_str())
            }
            serde_json::Value::Number(n) => {
                Self::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Array(arr) => {
                Self::Array(arr.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(obj) => Self::Object(
                obj.iter()
                    .map(|(k, v)| {
                        (k.clone().into_boxed_str(), Self::from_json(v))
                    })
                    .collect(),
            ),
            serde_json::Value::Null => Self::String("".into()),
        }
    }

    /// Convert this `FieldValue` to a JSON string for indexing.
    ///
    /// This provides a stable string representation for metadata indexes.
    #[inline]
    #[must_use]
    pub fn to_json_string(&self) -> String {
        match self {
            Self::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Self::Number(n) => n.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Date(ts) => ts.to_string(),
            Self::Array(arr) => {
                let elements: Vec<String> =
                    arr.iter().map(FieldValue::to_json_string).collect();
                format!("[{}]", elements.join(", "))
            }
            Self::Object(obj) => {
                let pairs: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_json_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn from_json_works() -> Result<(), Box<dyn std::error::Error>> {
        #[expect(
            clippy::disallowed_methods,
            reason = "json! macro is allowed in tests for convenience"
        )]
        let json = json!({
            "str": "hello",
            "num": 42.5f64,
            "bool": true,
            "arr": [1i32, "two"],
            "obj": {
                "nested": "val"
            },
            "null": null
        });

        let val = FieldValue::from_json(&json);

        let obj = val
            .as_object()
            .ok_or_else(|| "val should be an object".to_owned())?;

        assert_eq!(obj.get("str").and_then(FieldValue::as_str), Some("hello"));
        assert_eq!(
            obj.get("num").and_then(FieldValue::as_number),
            Some(42.5f64)
        );
        assert_eq!(obj.get("bool").and_then(FieldValue::as_bool), Some(true));

        let arr = obj
            .get("arr")
            .ok_or_else(|| "missing 'arr' key".to_owned())?
            .as_array()
            .ok_or_else(|| "'arr' should be an array".to_owned())?;

        assert_eq!(arr.first().and_then(FieldValue::as_number), Some(1.0f64));
        assert_eq!(arr.get(1).and_then(FieldValue::as_str), Some("two"));

        let nested = obj
            .get("obj")
            .ok_or_else(|| "missing 'obj' key".to_owned())?
            .as_object()
            .ok_or_else(|| "'obj' should be an object".to_owned())?;

        assert_eq!(
            nested.get("nested").and_then(FieldValue::as_str),
            Some("val")
        );

        // Null becomes empty string as per implementation
        assert_eq!(obj.get("null").and_then(FieldValue::as_str), Some(""));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn rkyv_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let val = FieldValue::Array(vec![
            FieldValue::String("test".into()),
            FieldValue::Number(123.0f64),
        ]);

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&val)?;
        let deserialized: FieldValue =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes)?;

        assert_eq!(val, deserialized);
        Ok(())
    }

    #[test]
    fn value_type_detection() {
        assert_eq!(
            FieldValue::String("".into()).value_type(),
            FieldValueType::String
        );
        assert_eq!(
            FieldValue::Number(0.0f64).value_type(),
            FieldValueType::Number
        );
        assert_eq!(
            FieldValue::Boolean(true).value_type(),
            FieldValueType::Boolean
        );
        assert_eq!(FieldValue::Date(0i64).value_type(), FieldValueType::Date);
        assert_eq!(
            FieldValue::Array(vec![]).value_type(),
            FieldValueType::Array
        );
        assert_eq!(
            FieldValue::Object(HashMap::new()).value_type(),
            FieldValueType::Object
        );
    }
}
