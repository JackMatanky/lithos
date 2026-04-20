//! Query-optimized view projections for note entities.
//!
//! This module contains persistable views that are derived from domain entities
//! and cached in the database for fast access. Views follow the "database as
//! cache" architectural principle - they can always be rebuilt from source
//! data.

#![expect(
    clippy::pub_use,
    reason = "Public API exposes ListView from note::views"
)]

pub mod list;

// Re-exports for public API ergonomics.
pub use list::ListView;
