//! `DbPathKey` redb trait implementations.

use std::ops::Deref;

use traces_fs::path::PathKey;

/// A newtype wrapper around `PathKey` to implement foreign database traits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbPathKey(PathKey);

impl DbPathKey {
    /// Constructs a database path key from a validated path string.
    ///
    /// # Errors
    ///
    /// Returns [`traces_fs::PathError`] when the input is not a valid path key.
    #[inline]
    pub fn try_new<T: AsRef<str>>(
        value: T,
    ) -> Result<Self, traces_fs::PathError> {
        PathKey::try_new(value.as_ref()).map(Self)
    }

    /// Unwraps the inner `PathKey`.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> PathKey {
        self.0
    }
}

impl Deref for DbPathKey {
    type Target = PathKey;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PathKey> for DbPathKey {
    #[inline]
    fn from(key: PathKey) -> Self {
        Self(key)
    }
}

impl From<&PathKey> for DbPathKey {
    #[inline]
    fn from(key: &PathKey) -> Self {
        Self(key.clone())
    }
}

impl redb::Value for DbPathKey {
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;
    type SelfType<'a>
        = DbPathKey
    where
        Self: 'a;

    #[inline]
    fn fixed_width() -> Option<usize> {
        None
    }

    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        #[expect(
            clippy::panic,
            reason = "redb::Value trait requires panic on invalid data"
        )]
        let Ok(s) = std::str::from_utf8(data) else {
            panic!("PathKey data from database must be valid UTF-8");
        };

        #[expect(
            clippy::panic,
            reason = "redb::Value trait requires panic on invalid data"
        )]
        let Ok(path_key) = PathKey::try_new(s) else {
            panic!("PathKey data from database must be normalized");
        };

        DbPathKey(path_key)
    }

    #[inline]
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> &'a [u8]
    where
        Self: 'a,
        Self: 'b,
    {
        value.0.as_str().as_bytes()
    }

    #[inline]
    fn type_name() -> redb::TypeName {
        redb::TypeName::new("traces::PathKey")
    }
}

impl redb::Key for DbPathKey {
    #[inline]
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}
