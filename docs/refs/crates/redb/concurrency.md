# Concurrency & Transactions in redb

Source: https://github.com/cberner/redb/blob/master/docs/design.md

## Transaction Model
redb supports multiple concurrent readers and a single writer.

## Durability Modes
- `Durability::Immediate`: Safest, calls `fsync` on commit.
- `Durability::Eventual`: Background `fsync`.
- `Durability::None`: Fastest, no durability guarantees.
