# Serialize in rkyv

## Derive Macro Serialize

[Source](https://docs.rs/rkyv/latest/rkyv/derive.Serialize.html)

```rust
#[derive(Serialize)]
{
    // Attributes available to this derive:
    #[rkyv]
}
```
Expand description

Derives `Serialize` for the labeled type.

This macro also supports the `#[rkyv]` attribute. See [`Archive`](https://docs.rs/rkyv/latest/rkyv/derive.Archive.html "derive rkyv::Archive") for more information.
