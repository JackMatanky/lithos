//! File property override types.

use crate::schema::aggregate::SchemaName;

/// File property override bundle.
///
/// All fields are `Option<T>` to support override contexts.
/// Inline definitions use `RawPropertyFile`.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::file::RawFileSpec;
///
/// let _spec = RawFileSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawFileSpec {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<Box<str>>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<SchemaName>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_file_spec_default() {
        let spec = RawFileSpec::default();
        assert_eq!(spec.directory, None);
        assert_eq!(spec.file_class, None);
    }

    #[test]
    fn raw_file_spec_serialize_with_fields() {
        let spec = RawFileSpec {
            directory: Some("docs/".into()),
            file_class: Some(SchemaName::try_new("document").unwrap()),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("directory"));
        assert!(json.contains("file_class"));
    }

    #[test]
    fn raw_file_spec_deserialize_with_fields() {
        let json = r#"{"directory": "docs/", "file_class": "document"}"#;
        let spec: RawFileSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.directory.as_deref(), Some("docs/"));
        assert_eq!(
            spec.file_class.as_ref().map(SchemaName::as_str),
            Some("document")
        );
    }

    #[test]
    fn raw_file_spec_deserialize_empty() {
        let json = "{}";
        let spec: RawFileSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.directory, None);
        assert_eq!(spec.file_class, None);
    }
}
