//! `PathKey` redb trait implementations.

use crate::fs::path::PathKey;

impl redb::Value for PathKey {
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;
    type SelfType<'a>
        = PathKey
    where
        Self: 'a;

    #[inline]
    fn fixed_width() -> Option<usize> {
        None // Variable-length UTF-8 strings
    }

    /// Deserialize `PathKey` from database bytes.
    ///
    /// # Panics
    /// Panics if stored data is not valid UTF-8 or violates `PathKey`
    /// normalization invariants. This indicates database corruption.
    ///
    /// This panic behavior matches redb ecosystem patterns (String, &str) and
    /// is required by the trait signature (no Result return type).
    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        #[expect(
            clippy::panic,
            reason = "redb::Value trait requires panic on invalid data - no \
                      Result return type allowed"
        )]
        let Ok(s) = std::str::from_utf8(data) else {
            panic!("PathKey data from database must be valid UTF-8");
        };

        #[expect(
            clippy::panic,
            reason = "redb::Value trait requires panic on invalid data - no \
                      Result return type allowed"
        )]
        let Ok(path_key) = PathKey::try_new(s) else {
            panic!("PathKey data from database must be normalized");
        };

        path_key
    }

    #[inline]
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> &'a [u8]
    where
        Self: 'a,
        Self: 'b,
    {
        value.as_str().as_bytes()
    }

    #[inline]
    fn type_name() -> redb::TypeName {
        redb::TypeName::new("lithos::PathKey")
    }
}

impl redb::Key for PathKey {
    /// Compare `PathKey` bytes lexicographically.
    ///
    /// UTF-8 byte comparison is valid because `PathKey` enforces UTF-8 at
    /// construction.
    #[inline]
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod serialization {
        use redb::Value;

        use super::*;

        #[test]
        fn preserves_value_across_redb_roundtrip() {
            let original =
                PathKey::try_new("notes/daily.md").expect("valid key");

            // Serialize via redb::Value
            let bytes = PathKey::as_bytes(&original);

            // Deserialize via redb::Value
            let deserialized = PathKey::from_bytes(bytes);

            assert_eq!(original, deserialized);
        }
    }

    mod ordering {
        use redb::Key;

        use super::*;

        #[test]
        fn orders_keys_lexicographically() {
            let key1 = PathKey::try_new("a/file.md").expect("valid");
            let key2 = PathKey::try_new("b/file.md").expect("valid");

            let bytes1 = key1.as_str().as_bytes();
            let bytes2 = key2.as_str().as_bytes();

            let result = PathKey::compare(bytes1, bytes2);

            assert_eq!(result, std::cmp::Ordering::Less);
        }
    }
}
