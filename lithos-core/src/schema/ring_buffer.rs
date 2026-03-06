//! Fixed-size ring buffer for versioned file storage.
//!
//! This ring buffer uses `u8` indices for memory efficiency (5 versions max).
//! All arithmetic and indexing is safe by design (modulo wraparound prevents
//! out-of-bounds).

#![expect(
    clippy::arithmetic_side_effects,
    reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "All indices are modulo N (cannot exceed array bounds)"
)]
#![expect(
    clippy::as_conversions,
    reason = "u8 <-> usize conversions safe for N <= 255 (enforced by use \
              case)"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "N is constrained to 5 in practice (ring buffer for file \
              versions)"
)]
#![expect(
    clippy::integer_division_remainder_used,
    reason = "Modulo operation is fundamental to ring buffer wraparound"
)]

use rkyv::{Archive, Deserialize, Serialize};

/// Fixed-size ring buffer (compile-time size, zero allocation).
///
/// # Constraints
/// - `N` should be small (≤ 255) to fit in `u8` indices
/// - Designed for 5-version file history (minimal memory overhead)
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RingBuffer<T, const N: usize> {
    items: [Option<T>; N],
    head: u8, // Next write position
    len: u8,  // Current count (0..=N)
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create empty ring buffer.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: [const { None }; N],
            head: 0,
            len: 0,
        }
    }

    /// Push item (evicts oldest if full).
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items[self.head as usize] = Some(item);
        self.head = (self.head + 1) % (N as u8);
        if self.len < N as u8 {
            self.len += 1;
        }
    }

    /// Get most recent item.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + (N as u8) - 1) % (N as u8);
        self.items[idx as usize].as_ref()
    }

    /// Get item at index (0 = oldest, len-1 = newest).
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len as usize {
            return None;
        }
        let offset =
            (self.head + (N as u8) - self.len + index as u8) % (N as u8);
        self.items[offset as usize].as_ref()
    }

    /// Number of items.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over items (oldest to newest).
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len()).filter_map(move |i| self.get(i))
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_push() {
        let mut buf = RingBuffer::<i32, 3>::new();
        assert_eq!(buf.len(), 0);

        buf.push(1i32);
        buf.push(2i32);
        buf.push(3i32);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.current(), Some(&3i32));
    }

    #[test]
    fn ring_buffer_eviction() {
        let mut buf = RingBuffer::<i32, 3>::new();
        buf.push(1i32);
        buf.push(2i32);
        buf.push(3i32);
        buf.push(4i32); // Evicts 1

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(&2i32)); // Oldest
        assert_eq!(buf.get(1), Some(&3i32));
        assert_eq!(buf.get(2), Some(&4i32)); // Newest
        assert_eq!(buf.current(), Some(&4i32));
    }
}
