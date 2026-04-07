//! Boolean property specification types.

/// Boolean property definition (marker type).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::spec_bool::RawBoolSpec;
///
/// let _spec = RawBoolSpec;
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[expect(
    clippy::exhaustive_structs,
    reason = "Marker type with no fields; non_exhaustive prevents construction"
)]
pub struct RawBoolSpec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_bool_spec_default() {
        let spec = RawBoolSpec;
        assert_eq!(spec, RawBoolSpec);
    }

    #[test]
    fn raw_bool_spec_serialize() {
        let spec = RawBoolSpec;
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn raw_bool_spec_deserialize() {
        let json = "null";
        let spec: RawBoolSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec, RawBoolSpec);
    }
}
