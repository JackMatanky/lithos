//! Core domain logic and infrastructure for the Lithos knowledge management
//! system.
//!
//! Organizes logic into bounded contexts (config, note, schema, template) with
//! zero-copy database primitives and secure filesystem utilities.
//!
//! Dependencies flow inward: cli → domain contexts → db → fs.

#![feature(trivial_bounds)]
#![recursion_limit = "1024"]

extern crate serde;

// Module declarations
pub mod application;
pub mod bounds;
pub mod config;
pub mod db;
pub mod fs;
pub mod note;
pub mod schema;
pub mod template;
pub mod vault;

/// Serialization utilities for rkyv integration.
pub mod ser {
    use rkyv::with::{ArchiveWith, DeserializeWith, SerializeWith};

    /// Helper for serializing `chrono::DateTime<Utc>` as i64.
    #[non_exhaustive]
    pub struct DateTimeAsI64;

    impl ArchiveWith<chrono::DateTime<chrono::Utc>> for DateTimeAsI64 {
        type Archived = rkyv::rend::i64_le;
        type Resolver = ();

        #[inline]
        fn resolve_with(
            field: &chrono::DateTime<chrono::Utc>,
            (): Self::Resolver,
            out: rkyv::Place<Self::Archived>,
        ) {
            out.write(rkyv::rend::i64_le::from_native(field.timestamp()));
        }
    }

    impl<S: rkyv::rancor::Fallible + rkyv::ser::Writer + ?Sized>
        SerializeWith<chrono::DateTime<chrono::Utc>, S> for DateTimeAsI64
    {
        #[inline]
        #[expect(
            clippy::little_endian_bytes,
            reason = "Explicit little-endian for cross-platform serialization"
        )]
        fn serialize_with(
            field: &chrono::DateTime<chrono::Utc>,
            serializer: &mut S,
        ) -> Result<Self::Resolver, S::Error> {
            serializer.write(
                rkyv::rend::i64_le::from_native(field.timestamp())
                    .to_native()
                    .to_le_bytes()
                    .as_slice(),
            )?;
            Ok(())
        }
    }

    impl<D: rkyv::rancor::Fallible + ?Sized>
        DeserializeWith<rkyv::rend::i64_le, chrono::DateTime<chrono::Utc>, D>
        for DateTimeAsI64
    {
        #[inline]
        fn deserialize_with(
            field: &rkyv::rend::i64_le,
            _: &mut D,
        ) -> Result<chrono::DateTime<chrono::Utc>, D::Error> {
            use chrono::TimeZone as _;
            Ok(chrono::Utc
                .timestamp_opt(field.to_native(), 0)
                .single()
                .unwrap_or_else(|| chrono::Utc.timestamp_nanos(0)))
        }
    }
}
