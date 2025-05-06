use enum_dispatch::enum_dispatch;

use crate::array::Array;
use crate::hyperloglog::HyperLogLog;
use crate::small::Small;
#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

/// Representation types supported by `CardinalityEstimator`
#[repr(u8)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[enum_dispatch]
pub(crate) enum Representation<const P: usize, const W: usize> {
    #[cfg_attr(feature = "with_serde", serde(rename = "s"))]
    Small(Small<P, W>),
    #[cfg_attr(feature = "with_serde", serde(rename = "a"))]
    Array(Array<P, W>),
    #[cfg_attr(feature = "with_serde", serde(rename = "h"))]
    Hll(HyperLogLog<P, W>),
}

/// Representation trait which must be implemented by all representations.
#[enum_dispatch(Representation<P, W>)]
pub(crate) trait RepresentationTrait<const P: usize, const W: usize> {
    fn insert_encoded_hash(&mut self, h: u32) -> Option<Representation<P, W>>;
    fn estimate(&self) -> usize;
    fn size_of(&self) -> usize;
    fn to_string(&self) -> String {
        format!("estimate: {}", self.estimate())
    }
}

impl<const P: usize, const W: usize> Representation<P, W> {
    pub fn iec(&mut self, h: u32) {
        if let Some(mut upgraded) = self.insert_encoded_hash(h) {
            std::mem::swap(self, &mut upgraded)
        }
    }
}

impl<const P: usize, const W: usize> Default for Representation<P, W> {
    fn default() -> Self {
        Representation::Small(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_size() {
        assert_eq!(std::mem::size_of::<Representation<0, 0>>(), 40);
    }
}
