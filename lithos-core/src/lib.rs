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
pub mod app;
pub mod bounds;
pub mod config;
pub mod db;
/// Vault discovery and boundary resolution.
pub(crate) mod discovery;
pub mod fs;
pub mod graph;
pub mod indexer;
pub mod note;
pub mod prelude;
pub mod schema;
pub(crate) mod support;
// pub mod template;  // TODO: rebuild from scratch
pub mod utils;
pub mod vault;
