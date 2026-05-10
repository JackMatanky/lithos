# redb Concurrency & Transactions

Source: https://github.com/cberner/redb/blob/master/docs/design.md

See [design.md](design.md) for a deep dive into redb's file format and commit strategies.

## Transaction Model
redb supports multiple concurrent readers and a single writer.

## Durability Modes
- `Durability::Immediate`: Safest, calls `fsync` on commit.
- `Durability::Eventual`: Background `fsync`.
- `Durability::None`: Fastest, no durability guarantees.
