//! Base schema domain type.
//!
//! Provides [`BaseSchema`], the phase-1 schema aggregate carrying a
//! file-local `extends` list (by [`SchemaName`]) prior to cross-schema
//! parent resolution.
//!
//! Relationship to other schema types:
//! - [`RawSchema`](super::raw::RawSchema) - on-disk input, fields named the
//!   same as the source file.
//! - [`BaseSchema`] - validated, name-based projection of the source file's
//!   `extends` and `excludes` lists. Multiple-inheritance ready.
//! - [`Schema`](super::aggregate::Schema) - fully resolved aggregate whose
//!   `parents` are [`SchemaId`]s after cross-schema lookup.
