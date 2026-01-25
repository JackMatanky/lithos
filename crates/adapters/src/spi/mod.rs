//! Service provider interface adapters.
//!
//! This module contains infrastructure utilities for implementing domain ports.

pub mod cache;
pub mod errors;
pub mod fs;

// Re-export parser types with descriptive names for convenience
pub use fs::parsers::{
    Dispatcher as ParserDispatcher, Json as JsonParser, Toml as TomlParser,
    Yaml as YamlParser,
};
