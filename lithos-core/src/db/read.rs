//! Read transaction wrapper.

#![allow(
    clippy::field_scoped_visibility_modifiers,
    dead_code,
    reason = "Exposing inner field for storage adapter access; dead code \
              until issue-02"
)]

/// Read-only transaction wrapper.
///
/// Provides scoped access to read-only database operations.
pub struct ReadTx {
    pub(crate) inner: redb::ReadTransaction,
}
