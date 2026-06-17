//! Command handler modules for the Lithos CLI.
//!
//! Each submodule corresponds to a top-level CLI subcommand and contains
//! the handler function, local port traits for testability, and unit tests.

pub(crate) mod config;
pub(crate) mod config_files;
pub(crate) mod doctor;
pub(crate) mod glue;
