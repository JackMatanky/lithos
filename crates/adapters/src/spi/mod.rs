//! Service provider interface adapters.
//!
//! This module contains infrastructure utilities for implementing domain ports.

pub mod errors;
pub mod parsers;

// Re-export parser types with descriptive names
pub use parsers::{
    Dispatcher as ParserDispatcher, Json as JsonParser, Toml as TomlParser,
    Yaml as YamlParser,
};
