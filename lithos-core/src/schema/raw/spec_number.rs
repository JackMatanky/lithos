//! Number property specification types.

/// Number property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::spec_number::RawNumberSpec;
///
/// let _spec = RawNumberSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawNumberSpec {
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_number_spec_default() {
        let spec = RawNumberSpec::default();
        assert_eq!(spec.max, None);
        assert_eq!(spec.min, None);
        assert_eq!(spec.step, None);
    }

    #[test]
    fn raw_number_spec_serialize_with_fields() {
        let spec = RawNumberSpec {
            max: Some(100.0f64),
            min: Some(0.0f64),
            step: Some(1.0f64),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("max"));
        assert!(json.contains("min"));
        assert!(json.contains("step"));
    }

    #[test]
    fn raw_number_spec_deserialize_with_fields() {
        let json = r#"{"max": 100, "min": 0, "step": 1}"#;
        let spec: RawNumberSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.max, Some(100.0f64));
        assert_eq!(spec.min, Some(0.0f64));
        assert_eq!(spec.step, Some(1.0f64));
    }

    #[test]
    fn raw_number_spec_deserialize_empty() {
        let json = "{}";
        let spec: RawNumberSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.max, None);
    }
}
