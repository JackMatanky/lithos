//! Indexer error types.

/// Errors that can occur during indexer operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IndexerError {
    /// An internal error with a descriptive message.
    #[error("internal error: {0}")]
    Internal(Box<str>),

    /// An I/O error from the underlying filesystem.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    mod formatting {
        use super::super::IndexerError;

        #[test]
        fn internal_error_displays_message() {
            let err = IndexerError::Internal("test failure".into());
            assert!(err.to_string().contains("test failure"));
        }

        #[test]
        fn io_error_displays_source_message() {
            let io = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file missing",
            );
            let err = IndexerError::Io(io);
            assert!(err.to_string().contains("file missing"));
        }

        #[test]
        fn internal_error_has_internal_prefix() {
            let err = IndexerError::Internal("oops".into());
            assert!(err.to_string().starts_with("internal error:"));
        }
    }
}
