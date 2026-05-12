//! Persistence infrastructure for rebuildable domain projections.
//!
//! This module provides a zero-copy database layer built on [`redb`] and
//! `rkyv`. It follows a "Storage-Adapter" pattern where domain repositories
//! use these primitives to persist and query state without direct coupling to
//! the underlying database implementation.
//!
//! # Core Concepts
//!
//! - **The Store**: The primary entry point ([`Store`]). It manages the
//!   physical database file and provides scoped, closure-based transactions.
//! - **Zero-Copy Reads**: Achieved by accessing archived data directly from
//!   database-mapped memory. Closure-based APIs ([`ReadTx`::`get` | `get_owned`
//!   ...]) ensure that references to this memory never outlive the transaction.
//! - **Type-Safe Tables**: Wrappers like [`UuidTable`] and [`PathTable`]
//!   enforce consistent key/value patterns across the codebase, preventing
//!   common bugs like mixing UUID strings and raw bytes.
//!
//! # Thread Safety & Concurrency
//!
//! - [`Store`] and [`Database`] are `Send + Sync` and can be shared across
//!   threads.
//! - `redb` supports multiple concurrent readers but only one writer.
//! - Write operations are automatically rolled back if the closure returns an
//!   error.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//!
//! use lithos_core::db::Store;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let store = Store::open(Path::new("my_data.db"))?;
//!
//! // Scoped read transaction
//! store.read(|tx| {
//!     // ... perform reads ...
//!     Ok(())
//! })?;
//! # Ok(())
//! # }
//! ```

mod core;
mod error;
mod read;
mod reader;
pub mod retry;
mod rkyv;
mod table;
mod uuid;
mod write;
mod writer;

pub use core::{Database, Store};

pub use error::{DbError, DbErrorKind};
pub use read::ReadTx;
pub use reader::BatchReader;
pub use rkyv::{deserialize, serialize, with_archived};
pub use table::{PathTable, Table, UuidMultimap, UuidTable};
pub use uuid::{
    UuidMultimapReadExt, UuidMultimapWriteExt, UuidTableReadExt,
    UuidTableWriteExt, UuidV7DbType, sealed,
};
pub use write::WriteTx;
pub use writer::BatchWriter;
