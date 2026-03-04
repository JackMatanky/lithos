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

    #[test]
    fn db_error_converts_from_redb_errors() {
        let db_err = redb::DatabaseError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        let result: DbError = db_err.into();
        assert!(
            result.to_string().contains("database error"),
            "Expected database error conversion message, got: {result}"
        );
    }
}
