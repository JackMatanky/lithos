//! Walkdir-based adapter implementing the
//! [`ScannerPort`](super::port::ScannerPort) trait.
//!
//! This module contains the concrete `WalkdirAdapter` that performs filesystem
//! traversal by wrapping the `walkdir` crate. It is the adapter layer — the
//! port contract lives in the parent `port` module.

mod walkdir;
