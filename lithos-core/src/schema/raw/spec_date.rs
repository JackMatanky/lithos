//! Date property specification types.

/// Date property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// (where `format` is required) and override contexts (where `None`
/// means "don't override").
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::spec_date::RawDateSpec;
///
/// let _spec = RawDateSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawDateSpec {
    /// Date format string (using chrono format tokens).
    pub format: Option<Box<str>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_date_spec_default() {
        let spec = RawDateSpec::default();
        assert_eq!(spec.format, None);
    }

    #[test]
    fn raw_date_spec_serialize_with_format() {
        let spec = RawDateSpec {
            format: Some("%Y-%m-%d".into()),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("format"));
    }

    #[test]
    fn raw_date_spec_deserialize_with_format() {
        let json = r#"{"format": "%Y-%m-%d"}"#;
        let spec: RawDateSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.format.as_deref(), Some("%Y-%m-%d"));
    }

    #[test]
    fn raw_date_spec_deserialize_empty() {
        let json = "{}";
        let spec: RawDateSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.format, None);
    }
}
