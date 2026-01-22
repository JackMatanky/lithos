//! Filesystem-related SPI utilities.
//!
//! This module contains file system infrastructure for Epic 4 (File Loading
//! Strategy Foundation):
//! - **parsers**: TOML/JSON/YAML parsing strategies
//! - **validator**: Path validation and security (Story 4.2)

pub mod parsers;
pub mod validator;
