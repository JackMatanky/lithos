//! Application composition root for Lithos.
//!
//! This module is the single place where core ports and adapters are wired
//! together and execution flows are exposed to executable adapters (e.g.
//! `lithos-cli`). It does not contain business logic; it composes the pieces
//! that own that logic.
//!
//! `lithos-core` remains a library crate — no `main.rs` lives here. The
//! process entrypoint remains `lithos-cli`, which calls into this module to
//! run any named execution flow.
//!
//! # Planned submodules
//!
//! - `commands`: typed app-level command structs (e.g. `IndexCommand`).
//! - `flows` (or `services`): execution flows such as `run_index` that
//!   orchestrate Discovery → Config → Indexer → routing.
//! - `composition`: construction of concrete adapters from runtime resources
//!   (database handle, filesystem root, etc.).
//! - `diagnostics`: app-level result summaries returned to executable adapters
//!   for rendering; not CLI-formatted output.

pub(crate) mod bootstrap;
