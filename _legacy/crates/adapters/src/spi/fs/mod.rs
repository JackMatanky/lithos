//! Filesystem-related SPI utilities.
//!
//! This module contains file system infrastructure for Epic 4 (File Loading
//! Strategy Foundation):
//!
//! ## Security-Critical Modules
//!
//! - **validator**: Path traversal protection and security validation (Story
//!   4.2)
//!   - Prevents path traversal attacks, symlink escapes, and arbitrary file
//!     access
//!   - Re-exported as `PathValidator` for ergonomic imports
//!
//! ## Data Processing Modules
//!
//! - **parsers**: TOML/JSON/YAML parsing strategies with auto-detection
//!   - Strategy pattern implementation for structured data formats
//!   - Re-exported as `FormatDispatcher` for clarity in calling code
//!
//! # Ergonomic Re-exports
//!
//! ```
//! use lithos_adapters::spi::fs::{FormatDispatcher, PathValidator};
//!
//! // Instead of:
//! // use lithos_adapters::spi::fs::validator::Validator;
//! // use lithos_adapters::spi::fs::parsers::Dispatcher;
//! ```

pub mod parsers;
pub mod validator;

// Ergonomic re-exports with domain-clarifying names
pub use parsers::Dispatcher as FormatDispatcher;
pub use validator::Validator as PathValidator;
