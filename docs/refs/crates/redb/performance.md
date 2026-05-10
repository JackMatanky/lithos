# redb Performance & Optimization

Source: https://docs.rs/redb/latest/redb/struct.AccessGuard.html

## Zero-Copy Reads
Access data directly via `AccessGuard`.

## insert_reserve
Avoid intermediate allocations for large values by reserving space and writing directly into the database page buffer.
