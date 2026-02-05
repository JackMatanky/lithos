/// Database error types.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
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

    /// Transaction failed.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Table operation failed.
    #[error("table error: {0}")]
    Table(String),
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
