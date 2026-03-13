//! View types for schema persistence.
//!
//! This module contains view types used for storage and queries.
//! Views are optional - only created when the domain type shape is
//! inefficient for storage or queries.
//!
//! ## Migration Status
//!
//! **Pending**: Types are being migrated from `storage.rs`:
//! - `Stored*` types → remain in storage.rs for now (used extensively)
//! - `Raw*File` types → to be moved to `raw.rs`
//! - Inheritance views → to be added to `inheritance.rs` if needed
//!
//! ## Future Structure
//!
//! - `raw.rs` - Raw file version storage (`RawSchemaFile`,
//!   `RawPropertyBankFile`, etc.)
//! - `inheritance.rs` - Inheritance relationship views (if profiling shows
//!   need)

/// Raw file version views.
///
/// **TODO**: Move `RawSchemaFile`, `RawPropertyBankFile`, `RawFileVersion`,
/// `FileChange`, `DecompressionError` from `storage.rs`.
pub mod raw;

/// Inheritance relationship views.
///
/// **TODO**: Create inheritance views if profiling shows they're needed for
/// performance. Current approach uses direct table queries.
pub mod inheritance;
