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

/// Centralised environment variable registry. All `LITHOS_*` and platform
/// environment variables are read once through lazy statics here. No other
/// module should call `std::env::var` or `std::env::var_os` for these keys.
pub mod dirs;

/// Vault discovery and boundary resolution.
pub mod discovery;
/// Resolved platform and application directories. Combines environment
/// overrides with platform defaults. Prefer this over calling the `dirs`
/// crate directly.
pub mod env;
pub mod fs;
pub mod graph;
pub mod indexer;
pub mod note;
pub mod prelude;
pub mod schema;
pub(crate) mod support;
pub mod template;
pub mod utils;
pub mod vault;
