#![feature(trivial_bounds)]
//! Application composition root for Lithos.
//!
//! This module is the single place where core ports and adapters are wired
//! together and execution flows are exposed to executable adapters (e.g.
//! `trace-cli`). It does not contain business logic; it composes the pieces
//! that own that logic.
//!
//! `trace-app` remains a library crate — no `main.rs` lives here. The
//! process entrypoint remains `trace-cli`, which calls into this module to
//! run any named execution flow.
//!
//! # Architecture
//!
//! - `index`: typed app-level commands (e.g. `IndexCommand`) and execution
//!   flows such as `run_index`. Composition is inline, orchestrating ports and
//!   concrete adapters from runtime resources (database handle, filesystem
//!   root, etc.).
//! - `bootstrap`: configuration and discovery pipeline setup.
//! - `error`: app-level error boundary.

pub mod bootstrap;
pub mod error;
pub mod index;
