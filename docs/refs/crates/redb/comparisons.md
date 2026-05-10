# redb Comparison with Alternatives

Source: https://github.com/cberner/redb

## redb vs SQLite
- **Pure Rust vs C dependency**: redb is written in pure Rust, making it easier to compile and cross-compile without a C toolchain. SQLite requires a C compiler and linking.
- **KV vs SQL**: redb is a key-value store, while SQLite is a relational database with SQL support. redb is better suited for simple KV needs where SQL overhead is unwanted.

## redb vs LMDB
- **Design Inspiration**: redb was loosely inspired by LMDB's design (copy-on-write B-trees, MVCC).
- **Pure Rust**: redb is pure Rust, whereas LMDB is written in C.
- **Safe API**: redb provides an idiomatic and memory-safe Rust API.

## redb vs Sled
- **Architecture**: redb uses copy-on-write B-trees; Sled uses a log-structured design.
- **Stability**: redb prioritizes file format stability.
- **Pure Rust**: Both are written in pure Rust.
