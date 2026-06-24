#!/bin/bash
cat << 'INNER_EOF' > /tmp/error.patch
--- crates/cli/src/error.rs
+++ crates/cli/src/error.rs
@@ -10,7 +10,8 @@
 //! - `2` — invalid explicit path or configuration error (user error)
 //! - `3` — filesystem permission denied or unreadable directory (I/O error)

+use std::path::PathBuf;
 use trace_app::error::AppError;
 use trace_settings::DiscoveryError;
+use trace_indexer::{IndexerError, ScannerError, error::IndexerRepositoryError};

 /// Top-level CLI error that wraps the bootstrap pipeline error.
@@ -34,6 +35,32 @@
     /// An invalid explicit path was provided.
     #[error("invalid path: {0}")]
     InvalidPath(String),
+
+    /// Error during the index operation.
+    #[error(transparent)]
+    Index(#[from] IndexCommandError),
+}
+
+#[derive(Debug, thiserror::Error, miette::Diagnostic)]
+pub(crate) enum IndexCommandError {
+    #[error("{path} does not exist")]
+    #[diagnostic(help("Provide a valid path, or omit --path to index the entire vault"))]
+    ScanPathNotFound { path: PathBuf },
+
+    #[error("cannot read {path}: permission denied")]
+    #[diagnostic(help("Grant read permission: chmod +r {path}"))]
+    ScanAccessDenied { path: PathBuf },
+
+    #[error("index database error: {detail}")]
+    #[diagnostic(help("Run `traces index --rebuild` to recreate the database"))]
+    StorageFailure { detail: String },
+
+    #[error("I/O error reading {path}: {detail}")]
+    #[diagnostic(help("Check disk space and filesystem health, then retry"))]
+    ScanIoError { path: PathBuf, detail: String },
+}
+
+impl From<IndexerError> for IndexCommandError {
+    fn from(err: IndexerError) -> Self {
+        match err {
+            IndexerError::Path(e) => IndexCommandError::ScanPathNotFound { path: PathBuf::from(e.to_string()) },
+            IndexerError::Scanner(ScannerError::Traversal { path, source }) => {
+                match source.kind() {
+                    std::io::ErrorKind::NotFound => IndexCommandError::ScanPathNotFound { path },
+                    std::io::ErrorKind::PermissionDenied => IndexCommandError::ScanAccessDenied { path },
+                    _ => IndexCommandError::ScanIoError { path, detail: source.to_string() },
+                }
+            }
+            IndexerError::Repository(IndexerRepositoryError::Storage(e)) => {
+                IndexCommandError::StorageFailure { detail: e.to_string() }
+            }
+            IndexerError::Repository(IndexerRepositoryError::DuplicatePath(p)) => {
+                IndexCommandError::StorageFailure { detail: format!("duplicate path: {}", p.as_str()) }
+            }
+        }
+    }
 }

 impl CliError {
@@ -52,8 +79,15 @@
             Self::Bootstrap(AppError::Discovery(discovery_err)) => {
                 exit_code_for_discovery(discovery_err)
             }
             Self::Bootstrap(AppError::Config(_)) | Self::InvalidPath(_) => 2,
-            Self::Bootstrap(AppError::Indexer(_))
-            | Self::Write {
+            Self::Bootstrap(AppError::Indexer(_)) => unreachable!("Indexer error should be mapped to IndexCommandError"),
+            Self::Write {
                 ..
             } => 3,
+            Self::Index(err) => match err {
+                IndexCommandError::ScanPathNotFound { .. } => 2,
+                IndexCommandError::ScanAccessDenied { .. } => 3,
+                IndexCommandError::StorageFailure { .. } => 2,
+                IndexCommandError::ScanIoError { .. } => 3,
+            },
         }
     }
 }
INNER_EOF
patch crates/cli/src/error.rs /tmp/error.patch
