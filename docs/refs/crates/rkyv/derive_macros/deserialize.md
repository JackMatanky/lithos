# Deserialize in rkyv

## Derive Macro Deserialize

[Source](https://docs.rs/rkyv/latest/rkyv/derive.Deserialize.html)

```rust
#[derive(Deserialize)]
{
    // Attributes available to this derive:
    #[rkyv]
}
```
Expand description

Derives `Deserialize` for the labeled type.

This macro also supports the `#[rkyv]` attribute. See [`Archive`](https://docs.rs/rkyv/latest/rkyv/derive.Archive.html "derive rkyv::Archive") for more information.
