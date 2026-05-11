//! Read transaction wrapper.

#![expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Exposing inner field for storage adapter access"
)]
#![allow(dead_code, reason = "dead code until issue-02")]

/// Read-only transaction wrapper.
///
/// Provides scoped access to read-only database operations within a
/// [`Store`](crate::db::Store).
pub struct ReadTx {
    pub(crate) inner: redb::ReadTransaction,
}
