# `walkdir` Crate: Struct `WalkDir`

Source: <https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html>

---

```rust
pub struct WalkDir { /* private fields */ }
```

A builder to create an iterator for recursively walking a directory.

Results are returned in depth first fashion, with directories yielded before their contents. If [`contents_first`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.contents_first) is true, contents are yielded before their directories. The order is unspecified but if [`sort_by`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.sort_by) is given, directory entries are sorted according to this function. Directory entries `.` and `..` are always omitted.

If an error occurs at any point during iteration, then it is returned in place of its corresponding directory entry and iteration continues as normal. If an error occurs while opening a directory for reading, then it is not descended into (but the error is still yielded by the iterator). Iteration may be stopped at any time. When the iterator is destroyed, all resources associated with it are freed.

## Usage

This type implements [`IntoIterator`](https://doc.rust-lang.org/stable/std/iter/trait.IntoIterator.html) so that it may be used as the subject of a `for` loop. You may need to call [`into_iter`](https://doc.rust-lang.org/nightly/core/iter/trait.IntoIterator.html#tymethod.into_iter) explicitly if you want to use iterator adapters such as [`filter_entry`](https://docs.rs/walkdir/latest/walkdir/struct.IntoIter.html#method.filter_entry).

Idiomatic use of this type should use method chaining to set desired options. For example, this only shows entries with a depth of `1`, `2` or `3` (relative to `foo`):

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("foo").min_depth(1).max_depth(3) {
    println!("{}", entry?.path().display());
}
```

Note that the iterator by default includes the top-most directory. Since this is the only directory yielded with depth `0`, it is easy to ignore it with the [`min_depth`](https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html#method.min_depth) setting:

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("foo").min_depth(1) {
    println!("{}", entry?.path().display());
}
```

This will only return descendents of the `foo` directory and not `foo` itself.

## Loops

This iterator (like most/all recursive directory iterators) assumes that no loops can be made with _hard_ links on your file system. In particular, this would require creating a hard link to a directory such that it creates a loop. On most platforms, this operation is illegal.

Note that when following symbolic/soft links, loops are detected and an error is reported.

## Implementations

### `impl WalkDir`

#### `pub fn new(root: impl AsRef<Path>) -> Self`

Create a builder for a recursive directory iterator starting at the file path `root`. If `root` is a directory, then it is the first item yielded by the iterator. If `root` is a file, then it is the first and only item yielded by the iterator. If `root` is a symlink, then it is always followed for the purposes of directory traversal. (A root `DirEntry` still obeys its documentation with respect to symlinks and the `follow_links` setting.)

#### `pub fn min_depth(self, depth: usize) -> Self`

Set the minimum depth of entries yielded by the iterator.

The smallest depth is `0` and always corresponds to the path given to the `new` function on this type. Its direct descendents have depth `1`, and their descendents have depth `2`, and so on.

#### `pub fn max_depth(self, depth: usize) -> Self`

Set the maximum depth of entries yield by the iterator.

The smallest depth is `0` and always corresponds to the path given to the `new` function on this type. Its direct descendents have depth `1`, and their descendents have depth `2`, and so on.

Note that this will not simply filter the entries of the iterator, but it will actually avoid descending into directories when the depth is exceeded.

#### `pub fn follow_links(self, yes: bool) -> Self`

Follow symbolic links. By default, this is disabled.

When `yes` is `true`, symbolic links are followed as if they were normal directories and files. If a symbolic link is broken or is involved in a loop, an error is yielded.

When enabled, the yielded [`DirEntry`](https://docs.rs/walkdir/latest/walkdir/struct.DirEntry.html) values represent the target of the link while the path corresponds to the link. See the [`DirEntry`](https://docs.rs/walkdir/latest/walkdir/struct.DirEntry.html) type for more details.

#### `pub fn follow_root_links(self, yes: bool) -> Self`

Follow symbolic links if these are the root of the traversal. By default, this is enabled.

When `yes` is `true`, symbolic links on root paths are followed which is effective if the symbolic link points to a directory. If a symbolic link is broken or is involved in a loop, an error is yielded as the first entry of the traversal.

When enabled, the yielded [`DirEntry`](https://docs.rs/walkdir/latest/walkdir/struct.DirEntry.html) values represent the target of the link while the path corresponds to the link. See the [`DirEntry`](https://docs.rs/walkdir/latest/walkdir/struct.DirEntry.html) type for more details, and all future entries will be contained within the resolved directory behind the symbolic link of the root path.

#### `pub fn max_open(self, n: usize) -> Self`

Set the maximum number of simultaneously open file descriptors used by the iterator.

`n` must be greater than or equal to `1`. If `n` is `0`, then it is set to `1` automatically. If this is not set, then it defaults to some reasonably low number.

This setting has no impact on the results yielded by the iterator (even when `n` is `1`). Instead, this setting represents a trade off between scarce resources (file descriptors) and memory. Namely, when the maximum number of file descriptors is reached and a new directory needs to be opened to continue iteration, then a previous directory handle is closed and has its unyielded entries stored in memory. In practice, this is a satisfying trade off because it scales with respect to the _depth_ of your file tree. Therefore, low values (even `1`) are acceptable.

Note that this value does not impact the number of system calls made by an exhausted iterator.

##### Platform behavior

On Windows, if `follow_links` is enabled, then this limit is not respected. In particular, the maximum number of file descriptors opened is proportional to the depth of the directory tree traversed.

#### `pub fn sort_by<F>(self, cmp: F) -> Self`

```rust
pub fn sort_by<F>(self, cmp: F) -> Self
where
    F: FnMut(&DirEntry, &DirEntry) -> Ordering + Send + Sync + 'static,
```

Set a function for sorting directory entries with a comparator function.

If a compare function is set, the resulting iterator will return all paths in sorted order. The compare function will be called to compare entries from the same directory.

```rust
use std::cmp;
use std::ffi::OsString;
use walkdir::WalkDir;

WalkDir::new("foo").sort_by(|a,b| a.file_name().cmp(b.file_name()));
```

#### `pub fn sort_by_key<F>(self, cmp: F) -> Self`

```rust
pub fn sort_by_key<F>(self, cmp: F) -> Self
where
    F: FnMut(&DirEntry) -> K + Send + Sync + 'static,
    K: Ord,
```

Set a function for sorting directory entries with a key extraction function.

If a compare function is set, the resulting iterator will return all paths in sorted order. The compare function will be called to compare entries from the same directory.

```rust
use std::cmp;
use std::ffi::OsString;
use walkdir::WalkDir;

WalkDir::new("foo").sort_by_key(|a| a.file_name().to_owned());
```

#### `pub fn sort_by_file_name(self) -> Self`

Sort directory entries by file name, to ensure a deterministic order.

This is a convenience function for calling `Self::sort_by()`.

```rust
use walkdir::WalkDir;

WalkDir::new("foo").sort_by_file_name();
```

#### `pub fn contents_first(self, yes: bool) -> Self`

Yield a directory’s contents before the directory itself. By default, this is disabled.

When `yes` is `false` (as is the default), the directory is yielded before its contents are read. This is useful when, e.g. you want to skip processing of some directories.

When `yes` is `true`, the iterator yields the contents of a directory before yielding the directory itself. This is useful when, e.g. you want to recursively delete a directory.

##### Example

Assume the following directory tree:

```
foo/
  abc/
    qrs
    tuv
  def/
```

With contents_first disabled (the default), the following code visits the directory tree in depth-first order:

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("foo") {
    let entry = entry.unwrap();
    println!("{}", entry.path().display());
}

// foo
// foo/abc
// foo/abc/qrs
// foo/abc/tuv
// foo/def
```

With contents_first enabled:

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("foo").contents_first(true) {
    let entry = entry.unwrap();
    println!("{}", entry.path().display());
}

// foo/abc/qrs
// foo/abc/tuv
// foo/abc
// foo/def
// foo
```

#### `pub fn same_file_system(self, yes: bool) -> Self`

Do not cross file system boundaries.

When this option is enabled, directory traversal will not descend into directories that are on a different file system from the root path.

Currently, this option is only supported on Unix and Windows. If this option is used on an unsupported platform, then directory traversal will immediately return an error and will not yield any entries.

## Trait Implementations

### `impl Debug for WalkDir`

#### `fn fmt(&self, f: &mut Formatter<'_>) -> Result`

Formats the value using the given formatter. [Read more](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt)

### `impl IntoIterator for WalkDir`

#### `type Item = Result<DirEntry, Error>`

The type of the elements being iterated over.

#### `type IntoIter = IntoIter`

Which kind of iterator are we turning this into?

#### `fn into_iter(self) -> Self::IntoIter`

Creates an iterator from a value. [Read more](https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.IntoIterator.html#tymethod.into_iter)

## Auto Trait Implementations

### `impl !RefUnwindSafe for WalkDir`

### `impl !UnwindSafe for WalkDir`

### `impl Freeze for WalkDir`

### `impl Send for WalkDir`

### `impl Sync for WalkDir`

### `impl Unpin for WalkDir`

### `impl UnsafeUnpin for WalkDir`

## Blanket Implementations

### `impl<T> Any for T`

```rust
impl<T> Any for T
where
    T: 'static + ?Sized,
```

#### `fn type_id(&self) -> TypeId`

Gets the `TypeId` of `self`. [Read more](https://doc.rust-lang.org/nightly/core/any/trait.Any.html#tymethod.type_id)

### `impl<T> Borrow<T> for T`

```rust
impl<T> Borrow<T> for T
where
    T: ?Sized,
```

#### `fn borrow(&self) -> &T`

Immutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html#tymethod.borrow)

### `impl<T> BorrowMut<T> for T`

```rust
impl<T> BorrowMut<T> for T
where
    T: ?Sized,
```

#### `fn borrow_mut(&mut self) -> &mut T`

Mutably borrows from an owned value. [Read more](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)

### `impl<T> From<T> for T`

```rust
impl<T> From<T> for T
```

#### `fn from(t: T) -> T`

Returns the argument unchanged.

### `impl<T, U> Into<U> for T`

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
```

#### `fn into(self) -> U`

Calls `U::from(self)`.

That is, this conversion is whatever the implementation of `From<T> for U` chooses to do.

### `impl<T, U> TryFrom<U> for T`

```rust
impl<T, U> TryFrom<U> for T
where
    U: Into<T>,
```

#### `type Error = Infallible`

The type returned in the event of a conversion error.

#### `fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>`

Performs the conversion.

### `impl<T, U> TryInto<U> for T`

```rust
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
```

#### `type Error = <U as TryFrom<T>>::Error`

The type returned in the event of a conversion error.

#### `fn try_into(self) -> Result<U, <U as TryFrom<T>>::Error>`

Performs the conversion.
