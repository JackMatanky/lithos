# Format Control & Compatibility

When you serialize a type with `rkyv`, the configuration chosen dictates the persistent byte format. Changing `rkyv` configuration flags later strictly results in breaking changes for all previously persisted data.

## Endianness
- Options: `little_endian` (default) or `big_endian`.
- Rule: Endianness must consistently match the target platform executing the deserialization (if optimizing for speed) or the established protocol standard. You cannot change this after data has been persisted without migrating the entire dataset.

## Alignment
- Options: `aligned` or `unaligned`.
- **`aligned`**: Runs faster in-memory as the CPU natively fetches aligned words. However, it *will crash* if an aligned struct is mapped onto a misaligned byte offset within a larger buffer.
- **`unaligned`**: Incurs a slight performance penalty but prevents alignment faults. **Highly recommended or mandatory** for memory-mapped files via the OS, since standard file buffering does not guarantee strict struct alignment.

## Pointer Width
- Options: `pointer_width_32` (default), `pointer_width_16`, `pointer_width_64`.
- This setting balances the serialized output size against the maximum addressable memory within the archive.
  - `32-bit`: Default. Usually the best balance.
  - `64-bit`: Required for humongous, multi-gigabyte datasets.
  - `16-bit`: Restricts archives to very small records but saves significant space per pointer.
