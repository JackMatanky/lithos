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
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Failed to open or create database.
    #[error(transparent)]
    Database(#[from] redb::DatabaseError),

    /// Commit operation failed.
    #[error(transparent)]
    Commit(#[from] redb::CommitError),

    /// Transaction operation failed.
    #[error(transparent)]
    Transaction(#[from] redb::TransactionError),

    /// Table operation failed.
    #[error(transparent)]
    Table(#[from] redb::TableError),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization or validation failed.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// TECHNICAL DEBT: Compatibility variant for mock testing and setup
    /// failures.
    ///
    /// # Note
    ///
    /// This variant is reserved for internal testing infrastructure (e.g.,
    /// tempdir failures, lock poisoning) and is slated for removal in favor
    /// of specialized test error types.
    #[error("database open failed: {0}")]
    Open(String),

    /// TECHNICAL DEBT: Data corruption or domain validation failure.
    ///
    /// # Warning
    ///
    /// This variant currently conflates low-level infrastructure failures
    /// (e.g., `redb::StorageError::Corrupted`) with high-level domain
    /// validation failures (e.g., invalid schema names).
    ///
    /// Domain-specific errors should ideally be handled at the Repository
    /// layer and not converted to a [`DbError`].
    #[error("data corruption: {0}")]
    Corruption(String),
}

impl DbError {
    /// Classify error for stable branching without backend-specific matching.
    ///
    /// Returns a stable [`DbErrorKind`] that callers can use for control flow
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
            Self::Commit(_) => DbErrorKind::Commit,
            Self::Transaction(_) => DbErrorKind::Transaction,
            Self::Table(_) => DbErrorKind::Table,
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
    /// See [`retry_on_transient`](crate::db::retry::retry_on_transient) for
    /// usage.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::db::DbError;
    ///
    /// let redb_err = redb::DatabaseError::from(std::io::Error::new(
    ///     std::io::ErrorKind::Other,
    ///     "database is locked",
    /// ));
    /// let io_error = DbError::Database(redb_err);
    /// // Note: actual transient detection depends on error message analysis
    /// assert!(io_error.is_transient());
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
            Self::Database(redb_err) => {
                let msg = redb_err.to_string().to_lowercase();
                msg.contains("locked")
                    || msg.contains("busy")
                    || msg.contains("i/o error")
                    || msg.contains("temporarily unavailable")
            }
            // Commit errors might be transient (I/O)
            Self::Commit(redb_err) => {
                let msg = redb_err.to_string().to_lowercase();
                msg.contains("locked")
                    || msg.contains("busy")
                    || msg.contains("i/o error")
                    || msg.contains("temporarily unavailable")
            }
            // Transaction errors might be transient (conflicts)
            Self::Transaction(redb_err) => {
                let msg = redb_err.to_string().to_lowercase();
                msg.contains("conflict") || msg.contains("locked")
            }
            // Table errors might be transient (I/O)
            Self::Table(redb_err) => {
                let msg = redb_err.to_string().to_lowercase();
                msg.contains("locked")
                    || msg.contains("busy")
                    || msg.contains("i/o error")
            }
            // All other errors are permanent (not retryable)
            Self::Corruption(_)
            | Self::Deserialization(_)
            | Self::Serialization(_)
            | Self::Open(_) => false,
        }
    }
}

impl From<redb::StorageError> for DbError {
    #[inline]
    fn from(e: redb::StorageError) -> Self {
        Self::Database(redb::DatabaseError::from(e))
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
            let redb_err =
                redb::DatabaseError::from(std::io::Error::other("test"));
            let err = DbError::Database(redb_err);
            assert_eq!(err.kind(), DbErrorKind::Database);
        }

        /// `DbError::kind()` returns `DbErrorKind::Transaction` for Transaction
        /// variant.
        #[test]
        fn transaction_variant_returns_transaction_kind() {
            let io_err = std::io::Error::other("test error");
            let storage_err = redb::StorageError::from(io_err);
            let err =
                DbError::Transaction(redb::TransactionError::from(storage_err));
            assert_eq!(err.kind(), DbErrorKind::Transaction);
        }

        /// `DbError::kind()` returns `DbErrorKind::Table` for Table variant.
        #[test]
        fn table_variant_returns_table_kind() {
            let io_err = std::io::Error::other("test error");
            let storage_err = redb::StorageError::from(io_err);
            let err = DbError::Table(redb::TableError::from(storage_err));
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

        /// Database locked errors are transient.
        #[test]
        fn database_locked_is_transient() {
            let redb_err = redb::DatabaseError::from(std::io::Error::other(
                "database is locked",
            ));
            let err = DbError::Database(redb_err);
            assert!(err.is_transient());
        }

        /// Transaction conflict errors are transient.
        #[test]
        fn transaction_conflict_is_transient() {
            let io_err = std::io::Error::other("transaction conflict");
            let storage_err = redb::StorageError::from(io_err);
            let err =
                DbError::Transaction(redb::TransactionError::from(storage_err));
            assert!(err.is_transient());
        }
    }

    /// Converting `redb::DatabaseError` to `DbError` preserves error metadata
    /// and classifies correctly.
    #[test]
    fn redb_database_error_converts_with_correct_kind() {
        let db_err = redb::DatabaseError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        let result: DbError = db_err.into();

        // Transparent wrapping means Display shows redb message directly
        assert!(
            !result.to_string().is_empty(),
            "Expected non-empty error message, got: {result}"
        );

        assert_eq!(result.kind(), DbErrorKind::Database);
    }

    /// `DbError::Database` wraps `redb::DatabaseError` transparently without
    /// string conversion.
    ///
    /// Behavior: Transparent wrapping preserves full error context via
    /// #[error(transparent)]. The Display impl delegates directly to the wrapped
    /// redb error, maintaining full diagnostic information.
    /// Verification: Create `redb::DatabaseError`, convert to `DbError`,
    /// verify Display output matches redb error (not generic wrapper message).
    #[test]
    fn database_error_wraps_redb_transparently() {
        let io_err =
            std::io::Error::new(std::io::ErrorKind::NotFound, "db not found");
        let redb_err = redb::DatabaseError::from(io_err);
        let redb_msg = redb_err.to_string();

        let db_err: DbError = redb_err.into();

        // Verify kind classification works
        assert_eq!(db_err.kind(), DbErrorKind::Database);

        // Verify transparent wrapping: Display should show redb error directly
        // (not "database error: <redb message>")
        let db_msg = db_err.to_string();
        assert_eq!(
            db_msg, redb_msg,
            "Expected transparent Display to match redb error message"
        );
    }

    /// `DbError::Commit` wraps `redb::CommitError` transparently.
    ///
    /// Behavior: Transparent wrapping for commit failures preserves full
    /// context via #[error(transparent)].
    /// Verification: Create `redb::CommitError`, convert to `DbError`,
    /// verify Display matches redb message and kind is Commit.
    #[test]
    fn commit_error_wraps_redb_transparently() {
        let io_err = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "commit failed",
        );
        let storage_err = redb::StorageError::from(io_err);
        let redb_err = redb::CommitError::from(storage_err);
        let redb_msg = redb_err.to_string();

        let db_err: DbError = redb_err.into();

        assert_eq!(db_err.kind(), DbErrorKind::Commit);

        let db_msg = db_err.to_string();
        assert_eq!(
            db_msg, redb_msg,
            "Expected transparent Display to match redb commit error"
        );
    }

    /// `DbError::Transaction` wraps `redb::TransactionError` transparently.
    #[test]
    fn transaction_error_wraps_redb_transparently() {
        let io_err = std::io::Error::other("transaction failed");
        let storage_err = redb::StorageError::from(io_err);
        let redb_err = redb::TransactionError::from(storage_err);
        let redb_msg = redb_err.to_string();

        let db_err: DbError = redb_err.into();

        assert_eq!(db_err.kind(), DbErrorKind::Transaction);
        assert_eq!(db_err.to_string(), redb_msg);
    }

    /// `DbError::Table` wraps `redb::TableError` transparently.
    #[test]
    fn table_error_wraps_redb_transparently() {
        let io_err = std::io::Error::other("table failed");
        let storage_err = redb::StorageError::from(io_err);
        let redb_err = redb::TableError::from(storage_err);
        let redb_msg = redb_err.to_string();

        let db_err: DbError = redb_err.into();

        assert_eq!(db_err.kind(), DbErrorKind::Table);
        assert_eq!(db_err.to_string(), redb_msg);
    }
}
