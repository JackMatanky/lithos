//! Lithos Core - Domain logic and infrastructure for the Lithos knowledge
//! management system.
//!
//! This crate provides the core domain logic with a dependency flow
//! architecture:
//! - Domain contexts (config, note, schema, template) contain business logic
//! - `db` provides zero-copy database primitives
//! - `fs` provides file system utilities
//!
//! Dependencies flow inward: cli → domain contexts → db → fs.

// Module declarations
pub mod config;
pub mod db;
pub mod fs;
pub mod note;
pub mod schema;
pub mod template;
