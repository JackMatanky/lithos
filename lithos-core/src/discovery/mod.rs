//! Discovery logic for establishing the context bounds.
//!
//! Provides the core primitives for locating and identifying root markers,
//! configuration files, and vault boundaries.

/// Discovery result contracts.
pub mod contracts;
/// Diagnostics and warnings during discovery.
pub mod diagnostics;
/// Root resolution logic.
pub mod resolver;

pub(crate) use contracts::*;
pub(crate) use diagnostics::*;
