#![allow(
    clippy::pattern_type_mismatch,
    reason = "Match on reference vs value in kind() method"
)]

/// Error classification for stable branching without backend-specific matching.
/// Provides a stable API for error handling that doesn't depend on
/// backend-specific error types. Use `DbError::kind()` to classify errors.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbErrorKind {
    /// Database-level operation failed (open, create, etc.).
    Database,
    /// Storage backend operation failed (I/O, corruption, etc.).
    Storage,
    /// Transaction operation failed (begin, commit, rollback, etc.).
    Transaction,
    /// Table operation failed (open, insert, get, etc.).
    Table,
    /// Commit operation failed.
    Commit,
    /// Serialization to bytes failed.
    Serialization,
    /// Deserialization from bytes failed.
    Deserialization,
}

/// Database error types.
#[non_exhaustive]
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum DbError {
    /// Database operation failed.
    #[error("database error: {0}")]
    Database(String),

    /// Database file not found or cannot be opened.
    #[error("failed to open database: {0}")]
    Open(String),

    /// Key not found in database.
    #[error("key not found")]
    NotFound,

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization or validation failed.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Data corruption detected.
    ///
    /// This error indicates that the database is in an inconsistent state,
    /// such as missing required metadata or referential integrity violations.
    #[error("data corruption detected: {0}")]
    Corruption(String),

    /// Transaction failed.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Table operation failed.
    #[error("table error: {0}")]
    Table(String),
}

impl DbError {
    /// Classify error for stable branching without backend-specific matching.
    ///
    /// Returns a stable error kind that callers can use for control flow
    /// without depending on backend-specific error types.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::db::{DbError, DbErrorKind};
    ///
    /// let err = DbError::Serialization("failed".into());
    /// assert_eq!(err.kind(), DbErrorKind::Serialization);
    /// ```
    #[inline]
    #[must_use]
    pub fn kind(&self) -> DbErrorKind {
        match self {
            Self::Database(_) | Self::Open(_) => DbErrorKind::Database,
            Self::Transaction(_) => DbErrorKind::Transaction,
            Self::Table(_) | Self::NotFound => DbErrorKind::Table,
            Self::Serialization(_) => DbErrorKind::Serialization,
            Self::Deserialization(_) => DbErrorKind::Deserialization,
            Self::Corruption(_) => DbErrorKind::Storage,
        }
    }

    /// Returns true if this error might be transient and worth retrying.
    ///
    /// Transient errors include:
    /// - Database locked/busy (concurrent access)
    /// - Temporary I/O errors
    /// - Some transaction conflicts
    ///
    /// Non-transient errors that should NOT be retried:
    /// - Corruption
    /// - Deserialization/validation errors
    /// - Missing data (`NotFound`)
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::db::DbError;
    ///
    /// let io_error = DbError::Database("database is locked".into());
    /// // Note: actual transient detection depends on error message analysis
    ///
    /// let corruption = DbError::Corruption("data corrupted".into());
    /// assert!(!corruption.is_transient());
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match pattern ergonomics preferred for readability"
    )]
    pub fn is_transient(&self) -> bool {
        match self {
            // Database errors might be transient (locked, I/O)
            Self::Database(msg) => {
                let lower = msg.to_lowercase();
                lower.contains("locked")
                    || lower.contains("busy")
                    || lower.contains("i/o error")
                    || lower.contains("temporarily unavailable")
            }
            // Transaction errors might be transient (conflicts)
            Self::Transaction(msg) => {
                let lower = msg.to_lowercase();
                lower.contains("conflict") || lower.contains("locked")
            }
            // All other errors are permanent (not retryable)
            Self::Corruption(_)
            | Self::Deserialization(_)
            | Self::Serialization(_)
            | Self::NotFound
            | Self::Open(_)
            | Self::Table(_) => false,
        }
    }
}

impl From<redb::DatabaseError> for DbError {
    #[inline]
    fn from(e: redb::DatabaseError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<redb::TransactionError> for DbError {
    #[inline]
    fn from(e: redb::TransactionError) -> Self {
        Self::Transaction(e.to_string())
    }
}

impl From<redb::TableError> for DbError {
    #[inline]
    fn from(e: redb::TableError) -> Self {
        Self::Table(e.to_string())
    }
}

impl From<redb::StorageError> for DbError {
    #[inline]
    fn from(e: redb::StorageError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<redb::CommitError> for DbError {
    #[inline]
    fn from(e: redb::CommitError) -> Self {
        Self::Transaction(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod kind {
        use super::*;

        /// `DbError::kind()` returns `DbErrorKind::Database` for Database
        /// variant.
        ///
        /// Behavior: Error classification via `kind()` provides stable API
        /// for callers to branch on error type without coupling to redb.
        /// Verification: Create `DbError::Database`, call `kind()`, assert
        /// returns `DbErrorKind::Database`.
        #[test]
        fn database_variant_returns_database_kind() {
            let err = DbError::Database("test".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Database);
        }

        /// `DbError::kind()` returns `DbErrorKind::Database` for Open variant.
        #[test]
        fn open_variant_returns_database_kind() {
            let err = DbError::Open("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Database);
        }

        /// `DbError::kind()` returns `DbErrorKind::Transaction` for Transaction
        /// variant.
        #[test]
        fn transaction_variant_returns_transaction_kind() {
            let err = DbError::Transaction("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Transaction);
        }

        /// `DbError::kind()` returns `DbErrorKind::Table` for Table variant.
        #[test]
        fn table_variant_returns_table_kind() {
            let err = DbError::Table("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Table);
        }

        /// `DbError::kind()` returns `DbErrorKind::Serialization` for
        /// Serialization variant.
        #[test]
        fn serialization_variant_returns_serialization_kind() {
            let err = DbError::Serialization("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Serialization);
        }

        /// `DbError::kind()` returns `DbErrorKind::Deserialization` for
        /// Deserialization variant.
        #[test]
        fn deserialization_variant_returns_deserialization_kind() {
            let err = DbError::Deserialization("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Deserialization);
        }

        /// `DbError::kind()` returns `DbErrorKind::Storage` for Corruption
        /// variant.
        #[test]
        fn corruption_variant_returns_storage_kind() {
            let err = DbError::Corruption("test error".to_owned());
            assert_eq!(err.kind(), DbErrorKind::Storage);
        }

        /// `DbError::kind()` returns `DbErrorKind::Table` for `NotFound`
        /// variant.
        #[test]
        fn notfound_variant_returns_table_kind() {
            let err = DbError::NotFound;
            assert_eq!(err.kind(), DbErrorKind::Table);
        }
    }

    mod is_transient {
        use super::*;

        /// Corruption errors are never transient.
        #[test]
        fn corruption_is_not_transient() {
            let err = DbError::Corruption("data corrupted".to_owned());
            assert!(!err.is_transient());
        }

        /// Deserialization errors are never transient.
        #[test]
        fn deserialization_is_not_transient() {
            let err = DbError::Deserialization("invalid data".to_owned());
            assert!(!err.is_transient());
        }

        /// Serialization errors are never transient.
        #[test]
        fn serialization_is_not_transient() {
            let err = DbError::Serialization("cannot serialize".to_owned());
            assert!(!err.is_transient());
        }

        /// `NotFound` errors are never transient.
        #[test]
        fn notfound_is_not_transient() {
            let err = DbError::NotFound;
            assert!(!err.is_transient());
        }

        /// Database locked errors are transient.
        #[test]
        fn database_locked_is_transient() {
            let err = DbError::Database("database is locked".to_owned());
            assert!(err.is_transient());
        }

        /// Transaction conflict errors are transient.
        #[test]
        fn transaction_conflict_is_transient() {
            let err = DbError::Transaction("transaction conflict".to_owned());
            assert!(err.is_transient());
        }
    }

    /// Converting `redb::DatabaseError` to `DbError` preserves error metadata.
    #[test]
    fn redb_database_error_converts_with_correct_kind() {
        let db_err = redb::DatabaseError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        let result: DbError = db_err.into();

        assert!(
            result.to_string().contains("database error"),
            "Expected database error conversion message, got: {result}"
        );

        assert_eq!(result.kind(), DbErrorKind::Database);
    }
}
