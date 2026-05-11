#![allow(
    clippy::pattern_type_mismatch,
    reason = "Match on reference vs value in kind() method"
)]
#![allow(
    clippy::missing_trait_methods,
    reason = "PartialEq::ne default implementation is correct for pointer \
              equality"
)]

use std::sync::Arc;

/// Wrapper around `redb::DatabaseError` that implements Clone/PartialEq/Eq.
///
/// Equality comparison uses pointer equality on the Arc (identity, not value).
#[derive(Debug, Clone)]
pub struct TransparentDatabaseError(Arc<redb::DatabaseError>);

impl PartialEq for TransparentDatabaseError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TransparentDatabaseError {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Display for TransparentDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for TransparentDatabaseError {
    type Target = redb::DatabaseError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper around `redb::TransactionError` that implements Clone/PartialEq/Eq.
#[derive(Debug, Clone)]
pub struct TransparentTransactionError(Arc<redb::TransactionError>);

impl PartialEq for TransparentTransactionError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TransparentTransactionError {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Display for TransparentTransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for TransparentTransactionError {
    type Target = redb::TransactionError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper around `redb::TableError` that implements Clone/PartialEq/Eq.
#[derive(Debug, Clone)]
pub struct TransparentTableError(Arc<redb::TableError>);

impl PartialEq for TransparentTableError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TransparentTableError {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Display for TransparentTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for TransparentTableError {
    type Target = redb::TableError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper around `redb::CommitError` that implements Clone/PartialEq/Eq.
#[derive(Debug, Clone)]
pub struct TransparentCommitError(Arc<redb::CommitError>);

impl PartialEq for TransparentCommitError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TransparentCommitError {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Display for TransparentCommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for TransparentCommitError {
    type Target = redb::CommitError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper around `redb::StorageError` that implements Clone/PartialEq/Eq.
#[derive(Debug, Clone)]
pub struct TransparentStorageError(Arc<redb::StorageError>);

impl PartialEq for TransparentStorageError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TransparentStorageError {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::fmt::Display for TransparentStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for TransparentStorageError {
    type Target = redb::StorageError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

    /// Database operation failed (transparent redb error wrapper).
    #[error("database error: {0}")]
    DatabaseTransparent(TransparentDatabaseError),

    /// Transaction failed (transparent redb error wrapper).
    #[error("transaction error: {0}")]
    TransactionTransparent(TransparentTransactionError),

    /// Table operation failed (transparent redb error wrapper).
    #[error("table error: {0}")]
    TableTransparent(TransparentTableError),

    /// Commit failed (transparent redb error wrapper).
    #[error("commit error: {0}")]
    CommitTransparent(TransparentCommitError),

    /// Storage operation failed (transparent redb error wrapper).
    #[error("storage error: {0}")]
    StorageTransparent(TransparentStorageError),
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
            Self::Database(_)
            | Self::Open(_)
            | Self::DatabaseTransparent(_) => DbErrorKind::Database,
            Self::Transaction(_) | Self::TransactionTransparent(_) => {
                DbErrorKind::Transaction
            }
            Self::Table(_) | Self::NotFound | Self::TableTransparent(_) => {
                DbErrorKind::Table
            }
            Self::CommitTransparent(_) => DbErrorKind::Commit,
            Self::Corruption(_) | Self::StorageTransparent(_) => {
                DbErrorKind::Storage
            }
            Self::Serialization(_) => DbErrorKind::Serialization,
            Self::Deserialization(_) => DbErrorKind::Deserialization,
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
            // Transparent database errors: check redb error type
            Self::DatabaseTransparent(err) => {
                // DatabaseAlreadyOpen and TransactionInProgress are transient
                // Storage errors might be transient (I/O)
                matches!(
                    **err,
                    redb::DatabaseError::DatabaseAlreadyOpen
                        | redb::DatabaseError::TransactionInProgress
                        | redb::DatabaseError::Storage(_)
                )
            }
            // Transparent transaction errors: check redb error type
            Self::TransactionTransparent(err) => {
                // ReadTransactionStillInUse is transient
                // Storage errors might be transient (I/O)
                matches!(
                    **err,
                    redb::TransactionError::ReadTransactionStillInUse(_)
                        | redb::TransactionError::Storage(_)
                )
            }
            // Transparent table errors: check for storage errors
            Self::TableTransparent(err) => {
                matches!(**err, redb::TableError::Storage(_))
            }
            // Transparent commit errors: check for storage errors
            Self::CommitTransparent(err) => {
                matches!(**err, redb::CommitError::Storage(_))
            }
            // Transparent storage errors: check for I/O
            Self::StorageTransparent(err) => {
                matches!(**err, redb::StorageError::Io(_))
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
        Self::DatabaseTransparent(TransparentDatabaseError(Arc::new(e)))
    }
}

impl From<redb::TransactionError> for DbError {
    #[inline]
    fn from(e: redb::TransactionError) -> Self {
        Self::TransactionTransparent(TransparentTransactionError(Arc::new(e)))
    }
}

impl From<redb::TableError> for DbError {
    #[inline]
    fn from(e: redb::TableError) -> Self {
        Self::TableTransparent(TransparentTableError(Arc::new(e)))
    }
}

impl From<redb::StorageError> for DbError {
    #[inline]
    fn from(e: redb::StorageError) -> Self {
        Self::StorageTransparent(TransparentStorageError(Arc::new(e)))
    }
}

impl From<redb::CommitError> for DbError {
    #[inline]
    fn from(e: redb::CommitError) -> Self {
        Self::CommitTransparent(TransparentCommitError(Arc::new(e)))
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

    mod transparent_wrappers {
        use super::*;

        /// `From<redb::DatabaseError>` wraps error transparently.
        ///
        /// Behavior: Converting `redb::DatabaseError` to `DbError` preserves
        /// the original error metadata (no string flattening).
        /// Verification: Convert `DatabaseError`, verify `kind()` returns
        /// Database, verify error can be downcasted back to redb type.
        #[test]
        fn database_error_wraps_transparently() {
            let redb_err = redb::DatabaseError::TransactionInProgress;
            let db_err: DbError = redb_err.into();

            // Verify kind classification works
            assert_eq!(db_err.kind(), DbErrorKind::Database);

            // Verify it's the transparent variant (not string-wrapped)
            assert!(
                matches!(db_err, DbError::DatabaseTransparent(_)),
                "Expected DbError::DatabaseTransparent variant, got: \
                 {db_err:?}"
            );
        }

        /// `From<redb::TransactionError>` wraps error transparently.
        #[test]
        fn transaction_error_wraps_transparently() {
            // Need to create a redb::TransactionError - use Storage variant
            let storage_err =
                redb::StorageError::Io(std::io::Error::other("test"));
            let redb_err = redb::TransactionError::Storage(storage_err);
            let db_err: DbError = redb_err.into();

            // Verify kind classification works
            assert_eq!(db_err.kind(), DbErrorKind::Transaction);

            // Verify it's the transparent variant
            assert!(
                matches!(db_err, DbError::TransactionTransparent(_)),
                "Expected DbError::TransactionTransparent variant, got: \
                 {db_err:?}"
            );
        }

        /// `From<redb::TableError>` wraps error transparently.
        #[test]
        fn table_error_wraps_transparently() {
            let storage_err =
                redb::StorageError::Io(std::io::Error::other("test"));
            let redb_err = redb::TableError::Storage(storage_err);
            let db_err: DbError = redb_err.into();

            assert_eq!(db_err.kind(), DbErrorKind::Table);
            assert!(
                matches!(db_err, DbError::TableTransparent(_)),
                "Expected DbError::TableTransparent variant, got: {db_err:?}"
            );
        }

        /// `From<redb::CommitError>` wraps error transparently.
        #[test]
        fn commit_error_wraps_transparently() {
            let storage_err =
                redb::StorageError::Io(std::io::Error::other("test"));
            let redb_err = redb::CommitError::Storage(storage_err);
            let db_err: DbError = redb_err.into();

            assert_eq!(db_err.kind(), DbErrorKind::Commit);
            assert!(
                matches!(db_err, DbError::CommitTransparent(_)),
                "Expected DbError::CommitTransparent variant, got: {db_err:?}"
            );
        }

        /// `From<redb::StorageError>` wraps error transparently.
        #[test]
        fn storage_error_wraps_transparently() {
            let redb_err =
                redb::StorageError::Io(std::io::Error::other("test"));
            let db_err: DbError = redb_err.into();

            assert_eq!(db_err.kind(), DbErrorKind::Storage);
            assert!(
                matches!(db_err, DbError::StorageTransparent(_)),
                "Expected DbError::StorageTransparent variant, got: {db_err:?}"
            );
        }
    }
}
