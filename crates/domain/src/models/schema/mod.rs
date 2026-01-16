//! Schema bounded context models.

pub mod core;
pub mod property;
pub mod registry;

#[cfg(test)]
pub use self::core::fixtures;
pub use self::{
    core::{DomainEvent, Schema},
    property::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, Property, PropertySpec,
        PropertySpecTrait, PropertySpecType, StringSpec,
    },
    registry::PropertyBank,
};
