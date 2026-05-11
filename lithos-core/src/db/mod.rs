//! Zero-copy database layer using redb and rkyv.
//!
//! This module provides concrete types (not traits) for database operations,
//! following the `std::fs::File` pattern. “Zero-copy” reads are achieved via
//! closure-based APIs that keep transactions properly scoped.
//!
//! # Architecture
//!
//! - [`Database`] - Concrete type wrapping `redb::Database`
//! - Closure-based API - Transactions scoped within closures (safe, no unsafe)
//! - Sync-first design - No async overhead
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//!
//! use lithos_core::db::Database;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Database::open(Path::new("/tmp/test.db"))?;
//! # Ok(())
//! # }
//! ```

#![allow(
    clippy::pub_use,
    clippy::impl_trait_in_params,
    reason = "This module intentionally re-exports a small public surface \
              (db::DbError, db::BatchWriter) for ergonomic crate consumers"
)]
#![allow(
    clippy::module_name_repetitions,
    reason = "DbError is intentionally explicit at the crate API boundary"
)]

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
pub use table::{PathTable, Table, UuidMultimap, UuidTable};
pub use uuid::UuidV7DbType;
pub use write::WriteTx;
pub use writer::BatchWriter;
