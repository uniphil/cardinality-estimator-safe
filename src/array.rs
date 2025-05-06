//! ## Array representation
//! Allows to estimate medium cardinality in [3..MAX_CAPACITY] range.
//!
//! The `data` format of array representation:
//! - 0..1 bits     - store representation type (bits are set to `01`)
//! - 2..55 bits    - store pointer to `u32` slice (on `x86_64` systems only 48-bits are needed).
//! - 56..63 bits   - store number of items `N` stored in array
//!
//! Slice encoding:
//! - data[0..N]    - store `N` encoded hashes
//! - data[N..]     - store zeros used for future hashes

use std::fmt::{Debug, Formatter};
use std::mem::size_of_val;
use std::ops::Deref;

use crate::hyperloglog::HyperLogLog;
use crate::representation::{Representation, RepresentationTrait};
#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

/// Maximum number of elements stored in array representation
pub(crate) const MAX_CAPACITY: usize = 128;

/// Array representation container
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
pub(crate) struct Array<const P: usize, const W: usize>(Vec<u32>);

impl<const P: usize, const W: usize> Array<P, W> {
    /// Insert encoded hash into `Array` representation
    /// Returns true on success, false otherwise.
    #[inline]
    pub(crate) fn insert(&mut self, h: u32) -> bool {
        if self.0.contains(&h) {
            return true;
        }
        if self.0.len() < MAX_CAPACITY {
            self.0.push(h);
            return true;
        }
        false
    }

    /// Create new instance of `Array` representation from vector
    #[inline]
    pub(crate) fn from_vec(arr: Vec<u32>, _len: usize) -> Array<P, W> {
        Self(arr)
    }
}

impl<const P: usize, const W: usize> RepresentationTrait<P, W> for Array<P, W> {
    /// Insert encoded hash into `HyperLogLog` representation.
    #[inline]
    fn insert_encoded_hash(&mut self, h: u32) -> Option<Representation<P, W>> {
        if self.insert(h) {
            None
        } else {
            // upgrade from `Array` to `HyperLogLog` representation
            let mut hll = HyperLogLog::<P, W>::new(self);
            hll.insert_encoded_hash(h);
            Some(Representation::Hll(hll))
        }
    }

    /// Return cardinality estimate of `Array` representation
    #[inline]
    fn estimate(&self) -> usize {
        self.0.len()
    }

    /// Return memory size of `Array` representation
    #[inline]
    fn size_of(&self) -> usize {
        size_of_val(self)
    }
}

impl<const P: usize, const W: usize> Debug for Array<P, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl<const P: usize, const W: usize> PartialEq for Array<P, W> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<const P: usize, const W: usize> Deref for Array<P, W> {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_size() {
        assert_eq!(std::mem::size_of::<Array<0, 0>>(), 24);
    }
}
