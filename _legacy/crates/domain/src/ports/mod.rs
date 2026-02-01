//! Domain ports module.
//!
//! This module defines trait interfaces for adapters to implement.

pub mod config;
/// Note domain ports.
pub mod note;
/// Schema domain ports.
pub mod schema;
/// SPI domain ports.
pub mod spi;
/// Template domain ports.
pub mod template;

pub use config::{Command as ConfigCommand, Query as ConfigQuery};
pub use note::{Command as NoteCommand, Query as NoteQuery};
pub use schema::{Command as SchemaCommand, Query as SchemaQuery};
pub use template::{Command as TemplateCommand, Query as TemplateQuery};
