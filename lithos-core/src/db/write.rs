//! Write transaction wrapper.

#![allow(
    clippy::field_scoped_visibility_modifiers,
    dead_code,
    reason = "Exposing inner field for storage adapter access; dead code \
              until issue-02"
)]

/// Read-write transaction wrapper.
///
/// Provides scoped access to read-write database operations within a [`Store`].
pub struct WriteTx {
    pub(crate) inner: redb::WriteTransaction,
}
