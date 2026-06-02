//! Discovery logic for establishing the context bounds.
//!
//! Provides the core primitives for locating and identifying root markers,
//! configuration files, and vault boundaries.

/// Diagnostics and warnings during discovery.
pub mod diagnostics;
/// Root marker contract: the typed output of vault root resolution.
pub mod marker;
/// Root resolution logic.
pub mod resolver;

pub(crate) use diagnostics::*;
pub(crate) use marker::*;
