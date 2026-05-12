# `walkdir` Crate: Struct `DirEntry`

[Source](https://docs.rs/walkdir/2.5.0/walkdir/struct.DirEntry.html)

```rust
pub struct DirEntry { /* private fields */ }
```

This is the type of value that is yielded from the iterators defined in this crate.

On Unix systems, this type implements the [`DirEntryExt`](https://docs.rs/walkdir/2.5.0/walkdir/trait.DirEntryExt.html) trait, which provides efficient access to the inode number of the directory entry.

## Differences with `std::fs::DirEntry`

This type mostly mirrors the type by the same name in [`std::fs`](https://doc.rust-lang.org/stable/std/fs/index.html). There are some differences however:

- All recursive directory iterators must inspect the entry’s type. Therefore, the value is stored and its access is guaranteed to be cheap and successful.
- [`path`](#method.path) and [`file_name`](#method.file_name) return borrowed variants.
- If [`follow_links`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.follow_links) was enabled on the originating iterator, then all operations except for [`path`](#method.path) operate on the link target. Otherwise, all operations operate on the symbolic link.

## Implementations

### `impl DirEntry`

The full path that this entry represents.

The full path is created by joining the parents of this entry up to the root initially given to [`WalkDir::new`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.new) with the file name of this entry.

Note that this _always_ returns the path reported by the underlying directory entry, even when symbolic links are followed. To get the target path, use [`path_is_symlink`](https://docs.rs/walkdir/2.5.0/walkdir/struct.DirEntry.html#method.path_is_symlink) to (cheaply) check if this entry corresponds to a symbolic link, and [`std::fs::read_link`](https://doc.rust-lang.org/stable/std/fs/fn.read_link.html) to resolve the target.

The full path that this entry represents.

Analogous to [`path`](https://docs.rs/walkdir/2.5.0/walkdir/struct.DirEntry.html#method.path), but moves ownership of the path.

Returns `true` if and only if this entry was created from a symbolic link. This is unaffected by the [`follow_links`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.follow_links) setting.

When `true`, the value returned by the [`path`](https://docs.rs/walkdir/2.5.0/walkdir/struct.DirEntry.html#method.path) method is a symbolic link name. To get the full target path, you must call [`std::fs::read_link(entry.path())`](https://doc.rust-lang.org/stable/std/fs/fn.read_link.html).

Return the metadata for the file that this entry points to.

This will follow symbolic links if and only if the [`WalkDir`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html) value has [`follow_links`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.follow_links) enabled.

##### Platform behavior

This always calls [`std::fs::symlink_metadata`](https://doc.rust-lang.org/stable/std/fs/fn.symlink_metadata.html).

If this entry is a symbolic link and [`follow_links`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.follow_links) is enabled, then [`std::fs::metadata`](https://doc.rust-lang.org/std/fs/fn.metadata.html) is called instead.

##### Errors

Similar to [`std::fs::metadata`](https://doc.rust-lang.org/std/fs/fn.metadata.html), returns errors for path values that the program does not have permissions to access or if the path does not exist.

Return the file type for the file that this entry points to.

If this is a symbolic link and [`follow_links`](https://docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.follow_links) is `true`, then this returns the type of the target.

This never makes any system calls.

Return the file name of this entry.

If this entry has no file name (e.g., `/`), then the full path is returned.

Returns the depth at which this entry was created relative to the root.

The smallest depth is `0` and always corresponds to the path given to the `new` function on `WalkDir`. Its direct descendents have depth `1`, and their descendents have depth `2`, and so on.

## Trait Implementations

### impl Clone for DirEntry

Returns a duplicate of the value. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone)

#### fn clone_from(&mut self, source: &Self)

Performs copy-assignment from `source`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from)

### impl Debug for DirEntry

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

### impl DirEntryExt for DirEntry

Available on **Unix** only.

Returns the underlying `d_ino` field in the contained `dirent` structure.

## Auto Trait Implementations

## Blanket Implementations

[Source](https://doc.rust-lang.org/nightly/src/core/any.rs.html#138)

### impl<T> Any for Twhere T: 'static +?Sized,

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#212)

### impl<T> Borrow<T> for Twhere T:?Sized,

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#221)

### impl<T> BorrowMut<T> for Twhere T:?Sized,

[Source](https://doc.rust-lang.org/nightly/src/core/borrow.rs.html#222)

#### fn borrow_mut(&mut self) -> &mut T

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

[Source](https://doc.rust-lang.org/nightly/src/core/clone.rs.html#515)

### impl<T> CloneToUninit for Twhere T: Clone,

🔬This is a nightly-only experimental API. (`clone_to_uninit`)

Performs copy-assignment from `self` to `dest`. [Read more](https://doc.rust-lang.org/nightly/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#785)

### impl<T> From<T> for T

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#788)

#### fn from(t: T) -> T

Returns the argument unchanged.

### impl<T, U> Into<U> for Twhere U: From<T>,

[Source](https://doc.rust-lang.org/nightly/src/core/convert/mod.rs.html#777)

#### fn into(self) -> U

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

### impl<T> ToOwned for Twhere T: Clone,

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#89)

#### type Owned = T

The resulting type after obtaining ownership.

[Source](https://doc.rust-lang.org/nightly/src/alloc/borrow.rs.html#90)

#### fn to_owned(&self) -> T

Creates owned data from borrowed data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)

Uses borrowed data to replace owned data, usually by cloning. [Read more](https://doc.rust-lang.org/nightly/alloc/borrow/trait.ToOwned.html#method.clone_into)

### impl<T, U> TryFrom<U> for Twhere U: Into<T>,

The type returned in the event of a conversion error.

Performs the conversion.

### impl<T, U> TryInto<U> for Twhere U: TryFrom<T>,

The type returned in the event of a conversion error.

Performs the conversion.
