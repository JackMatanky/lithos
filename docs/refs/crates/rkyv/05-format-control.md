# Format Control & Compatibility

When you serialize a type with `rkyv`, the configuration chosen dictates the persistent byte format. Changing `rkyv` configuration flags later strictly results in breaking changes for all previously persisted data.

**Ref:** [rkyv Format Control Guide](https://rkyv.org/format.html)
**Ref:** [Format Control Features](https://docs.rs/rkyv/latest/rkyv/#format-control)

## Endianness
- Options: `little_endian` (default) or `big_endian`.
- Rule: Endianness must consistently match the target platform executing the deserialization (if optimizing for speed) or the established protocol standard. You cannot change this after data has been persisted without migrating the entire dataset.

```toml
[dependencies]
rkyv = { version = "0.8", features = ["little_endian"] }
```

## Alignment
**Ref:** [Alignment Deep-Dive](https://rkyv.org/format/alignment.html)

- Options: `aligned` or `unaligned`.
- **`aligned`** (Default): Runs faster in-memory as the CPU natively fetches aligned words. However, it *will crash* if an aligned struct is mapped onto a misaligned byte offset within a larger buffer.
- **`unaligned`**: Incurs a slight performance penalty but prevents alignment faults. **Highly recommended or mandatory** for memory-mapped files via the OS, since standard file buffering does not guarantee strict struct alignment.

```toml
[dependencies]
rkyv = { version = "0.8", features = ["unaligned"] }
```

## Pointer Width
- Options: `pointer_width_32` (default), `pointer_width_16`, `pointer_width_64`.
- This setting balances the serialized output size against the maximum addressable memory within the archive.
  - `32-bit`: Default. Usually the best balance.
  - `64-bit`: Required for humongous, multi-gigabyte datasets.
  - `16-bit`: Restricts archives to very small records but saves significant space per pointer.

```toml
[dependencies]
rkyv = { version = "0.8", features = ["pointer_width_32"] }
```

## Lithos Guideline: Format Contracts
In Lithos, we treat format control features as an immutable contract for `redb` storage. Once a database schema is finalized, changing alignment or endianness will instantly corrupt existing user databases.
- Always ensure `unaligned` is used if memory mapping buffers from disk, as standard OS reads cannot guarantee struct alignment.
