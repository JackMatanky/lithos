//! Database, codec, and storage error types.
//!
//! The primary entry points are [`DbError`] (full error context with source
//! chain) and [`DbErrorKind`] (stable classification for branching). Codec
//! operations have a dedicated sub-type: [`CodecError`] with classification
//! [`CodecErrorKind`].
//!
//! # Quick Start
//!
//! ```rust
//! use traces_db::{DbError, DbErrorKind};
//!
//! let err = DbError::Corruption("data corrupted".into());
//! assert_eq!(err.kind(), DbErrorKind::Storage);
//! assert!(!err.is_transient());
//! ```

/// Error classification for stable branching without backend-specific matching.
///
/// Provides a stable API for error handling that doesn't depend on
/// backend-specific error types. Use [`DbError::kind`] to classify errors.
///
/// # Examples
///
/// ```
/// use traces_db::{DbError, DbErrorKind};
///
/// let err = DbError::Corruption("corrupt data".into());
/// assert_eq!(err.kind(), DbErrorKind::Storage);
/// ```
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    /// Codec operation failed.
    Codec,
}

/// Database error type with full source chain.
///
/// Each variant preserves the underlying error source. Use
/// [`kind`](DbError::kind) for classification without backend-specific matching
/// and [`is_transient`](DbError::is_transient) for retry decisions.
///
/// # Variants
///
/// | Category | Variants | Backend |
/// |---|---|---|
/// | Backend errors | `Database`, `Commit`, `Transaction`, `Table`, `Storage` | [`redb`] |
/// | Codec errors | `Codec` | rkyv |
/// | Compatibility | `Corruption` | — |
///
/// # Examples
///
/// ```
/// use traces_db::{DbError, DbErrorKind};
///
/// let err = DbError::Corruption("data corrupted".into());
/// assert_eq!(err.kind(), DbErrorKind::Storage);
/// assert!(!err.is_transient());
/// ```
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Database-level error (open, create, I/O).
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

    /// Storage operation failed.
    #[error(transparent)]
    Storage(#[from] redb::StorageError),

    /// Codec operation failed.
    #[error(transparent)]
    Codec(#[from] CodecError),

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
    /// use traces_db::{DbError, DbErrorKind};
    ///
    /// let err = DbError::Corruption("test error".into());
    /// assert_eq!(err.kind(), DbErrorKind::Storage);
    /// ```
    #[inline]
    #[must_use]
    pub fn kind(&self) -> DbErrorKind {
        match self {
            Self::Database(_) => DbErrorKind::Database,
            Self::Storage(_) | Self::Corruption(_) => DbErrorKind::Storage,
            Self::Commit(_) => DbErrorKind::Commit,
            Self::Transaction(_) => DbErrorKind::Transaction,
            Self::Table(_) => DbErrorKind::Table,
            Self::Codec(_) => DbErrorKind::Codec,
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
    ///
    /// See [`retry_on_transient`](crate::retry::retry_on_transient) for
    /// usage.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_db::DbError;
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
    #[allow(
        deprecated,
        reason = "legacy variants remain permanent during migration"
    )]
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
            // Storage errors might be transient (I/O)
            Self::Storage(redb_err) => {
                let msg = redb_err.to_string().to_lowercase();
                msg.contains("locked")
                    || msg.contains("busy")
                    || msg.contains("i/o error")
            }
            // All other errors are permanent (not retryable)
            Self::Corruption(_) | Self::Codec(_) => false,
        }
    }
}

/// Error classification for codec failures.
///
/// A sub-type of the error hierarchy — use [`CodecError::kind`] for
/// codec-specific branching, or [`DbError::kind`] for general classification
/// (returns [`DbErrorKind::Codec`]).
///
/// # Examples
///
/// ```
/// use traces_db::{CodecError, CodecErrorKind};
/// # use rkyv::rancor::Source;
///
/// let err = CodecError::RkyvSerialize {
///     type_name: "String",
///     source: rkyv::rancor::Error::new(std::io::Error::other("test")),
/// };
/// assert_eq!(err.kind(), CodecErrorKind::Encode);
/// ```
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodecErrorKind {
    /// Serialization to bytes failed.
    Encode,
    /// Archived byte validation failed.
    Access,
    /// Deserialization from archived bytes failed.
    Decode,
}

/// Errors from rkyv codec operations.
///
/// A sub-type of the error hierarchy — wrap in [`DbError::Codec`] to
/// integrate with the general [`DbError`] system.
///
/// # Examples
///
/// ```
/// use traces_db::{CodecError, CodecErrorKind};
/// # use rkyv::rancor::Source;
///
/// let err = CodecError::RkyvSerialize {
///     type_name: "String",
///     source: rkyv::rancor::Error::new(std::io::Error::other("test")),
/// };
/// assert_eq!(err.kind(), CodecErrorKind::Encode);
/// ```
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// Serialization to rkyv bytes failed.
    #[error("failed to serialize {type_name} with rkyv")]
    RkyvSerialize {
        /// Rust type being serialized.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },

    /// Archived byte validation failed.
    #[error("failed to validate archived {type_name}")]
    RkyvAccess {
        /// Rust type being validated.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },

    /// Deserialization from archived bytes failed.
    #[error("failed to deserialize archived {type_name}")]
    RkyvDeserialize {
        /// Rust type being deserialized.
        type_name: &'static str,
        /// Underlying rkyv error.
        #[source]
        source: rkyv::rancor::Error,
    },
}

impl CodecError {
    /// Classify codec error for stable branching.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_db::{CodecError, CodecErrorKind};
    /// # use rkyv::rancor::Source;
    ///
    /// let err = CodecError::RkyvSerialize {
    ///     type_name: "String",
    ///     source: rkyv::rancor::Error::new(std::io::Error::other("test")),
    /// };
    /// assert_eq!(err.kind(), CodecErrorKind::Encode);
    /// ```
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> CodecErrorKind {
        match self {
            Self::RkyvSerialize {
                ..
            } => CodecErrorKind::Encode,
            Self::RkyvAccess {
                ..
            } => CodecErrorKind::Access,
            Self::RkyvDeserialize {
                ..
            } => CodecErrorKind::Decode,
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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
