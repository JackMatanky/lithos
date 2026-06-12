//! Table definitions for template storage.
//!
//! These constants define the tables used by the `redb`-backed
//! [`RedbRepository`] to persist template aggregates and raw template views.
//!
//! [`RedbRepository`]: super::RedbRepository

use crate::{
    db::{PathTable, Table, UuidTable},
    impl_redb_uuid,
    template::aggregate::TemplateId,
};

impl_redb_uuid!(TemplateId);

/// Template aggregates with zero-copy serialization.
///
/// Stores full `Template` structures indexed by template ID for efficient
/// retrieval by identity.
///
/// Key: `TemplateId`
/// Value: rkyv-serialized `Template`
#[allow(dead_code, reason = "used by read.rs/write.rs trait impls")]
pub(crate) const TEMPLATES: UuidTable<TemplateId, &[u8]> =
    UuidTable::new("templates");

/// Template name-to-ID index for fast name-based lookup.
///
/// Enables resolving template IDs by name without loading full template data.
/// Maintained atomically with `TEMPLATES` during save operations.
///
/// Key: template name string
/// Value: serialized `TemplateId`
#[allow(dead_code, reason = "used by read.rs/write.rs trait impls")]
pub(crate) const TEMPLATE_ID_BY_NAME: Table<&str, &[u8]> =
    Table::new("template_id_by_name");

/// Raw template views for staleness detection.
///
/// Maps vault-relative paths to their raw template views, enabling change
/// detection without re-parsing the full template content.
///
/// Key: path string
/// Value: serialized `RawTemplateView`
#[allow(dead_code, reason = "used by read.rs/write.rs trait impls")]
pub(crate) const RAW_TEMPLATE_VIEWS: PathTable<&[u8]> =
    PathTable::new("raw_template_views");
