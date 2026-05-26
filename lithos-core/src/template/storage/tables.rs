//! Template storage table definitions and typed wrappers.

use crate::{
    db::{Table, UuidTable},
    impl_redb_uuid,
    template::aggregate::TemplateId,
};

impl_redb_uuid!(TemplateId);

/// Table mapping template ID to serialized template aggregate.
///
/// Key: `TemplateId` (UUID v7)
/// Value: rkyv-serialized `Template`
pub(crate) const TEMPLATES: UuidTable<TemplateId, &[u8]> =
    UuidTable::new("templates");

/// Map mapping template name to template ID.
///
/// Key: `TemplateName` string
/// Value: `TemplateId` (UUID v7)
pub(crate) const NAME_TO_ID: Table<&str, TemplateId> = Table::new("name_to_id");
