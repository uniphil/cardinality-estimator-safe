//! ## Small representation
//! Allows to estimate cardinality in [0..2] range and uses only 8 bytes of memory.
//!
//! The `data` format of small representation:
//! - 0..1 bits     - store representation type (bits are set to `00`)
//! - 2..33 bits    - store 31-bit encoded hash
//! - 34..63 bits   - store 31-bit encoded hash

use std::fmt::{Debug, Formatter};

use crate::array::Array;
use crate::representation::{Representation, RepresentationTrait};
#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

/// Mask used for extracting hashes stored in small representation (31 bits)
const SMALL_MASK: u64 = 0x0000_0000_7fff_ffff;

/// Small representation container
#[derive(PartialEq, Default)]
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
pub(crate) struct Small<const P: usize, const W: usize>(u64);

impl<const P: usize, const W: usize> Small<P, W> {
    /// Insert encoded hash into `Small` representation.
    /// Returns true on success, false otherwise.
    #[inline]
    pub(crate) fn insert(&mut self, h: u32) -> bool {
        let h1 = self.h1();
        if h1 == 0 {
            self.0 |= u64::from(h) << 2;
            return true;
        } else if h1 == h {
            return true;
        }

        let h2 = self.h2();
        if h2 == 0 {
            self.0 |= u64::from(h) << 33;
            return true;
        } else if h2 == h {
            return true;
        }

        false
    }

    /// Return 1-st encoded hash
    #[inline]
    fn h1(&self) -> u32 {
        ((self.0 >> 2) & SMALL_MASK) as u32
    }

    /// Return 2-nd encoded hash
    #[inline]
    fn h2(&self) -> u32 {
        ((self.0 >> 33) & SMALL_MASK) as u32
    }

    /// Return items stored within `Small` representation
    #[inline]
    pub(crate) fn items(&self) -> [u32; 2] {
        [self.h1(), self.h2()]
    }
}

impl<const P: usize, const W: usize> RepresentationTrait<P, W> for Small<P, W> {
    /// Insert encoded hash into `Small` representation.
    fn insert_encoded_hash(&mut self, h: u32) -> Option<Representation<P, W>> {
        if self.insert(h) {
            None
        } else {
            // upgrade from `Small` to `Array` representation
            let arr = Array::<P, W>::from_small(self.h1(), self.h2(), h);
            // let [a, b] = self.items();
            // let arr = Array::<P, W>::from_vec(vec![self.h1(), self.h2(), h], 3);
            Some(Representation::Array(arr))
        }
    }

    /// Return cardinality estimate of `Small` representation
    #[inline]
    fn estimate(&self) -> usize {
        match (self.h1(), self.h2()) {
            (0, 0) => 0,
            (_, 0) => 1,
            (_, _) => 2,
        }
    }

    /// Return memory size of `Small` representation
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl<const P: usize, const W: usize> Debug for Small<P, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl<const P: usize, const W: usize> From<u64> for Small<P, W> {
    /// Create new instance of `Small` from given `data`
    fn from(data: u64) -> Self {
        Self(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_size() {
        assert_eq!(std::mem::size_of::<Small<0, 0>>(), 8);
    }
}
