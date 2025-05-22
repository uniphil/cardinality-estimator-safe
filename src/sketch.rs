use std::ops::Deref;
use enum_dispatch::enum_dispatch;

use crate::array::Array;
use crate::hyperloglog::HyperLogLog;
use crate::small::Small;
use crate::element::Element;
#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

/// Sketch types supported by `CardinalityEstimator`
#[repr(u8)]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[enum_dispatch]
#[allow(private_interfaces)]
pub enum Sketch<const P: usize = 12, const W: usize = 6> {
    #[cfg_attr(feature = "with_serde", serde(rename = "s"))]
    Small(Small<P, W>),
    #[cfg_attr(feature = "with_serde", serde(rename = "a"))]
    Array(Array<P, W>),
    #[cfg_attr(feature = "with_serde", serde(rename = "h"))]
    Hll(HyperLogLog<P, W>),
}

/// Sketch trait which must be implemented by all representations.
#[enum_dispatch(Sketch<P, W>)]
pub(crate) trait SketchTrait<const P: usize, const W: usize> {
    fn insert_encoded_hash(&mut self, h: u32) -> Option<Sketch<P, W>>;
    fn estimate_sketch(&self) -> usize;
    #[allow(dead_code)]
    fn size_of(&self) -> usize;
    fn to_string(&self) -> String {
        format!("estimate: {}", self.estimate_sketch())
    }
}

impl<const P: usize, const W: usize> Sketch<P, W> {
    pub fn insert(&mut self, element: Element<P, W>) {
        self.insert_encoded(element.0)
    }

    pub fn estimate(&self) -> usize {
        self.estimate_sketch()
    }

    #[inline]
    fn insert_encoded(&mut self, encoded: u32) {
        if let Some(upgraded) = self.insert_encoded_hash(encoded) {
            *self = upgraded;
        }
    }

    /// Merge cardinality estimators
    #[inline]
    pub fn merge(&mut self, rhs: &Self) {
        match &rhs {
            Sketch::Small(rhs_small) => {
                for h in rhs_small.items() {
                    if h != 0 {
                        self.insert_encoded(h);
                    }
                }
            }
            Sketch::Array(rhs_arr) => {
                for &h in rhs_arr.deref() {
                    self.insert_encoded(h);
                }
            }
            Sketch::Hll(rhs_hll) => {
                match self {
                    Sketch::Small(lhs_small) => {
                        let mut hll = rhs_hll.clone();
                        for h in lhs_small.items() {
                            if hll.insert_encoded_hash(h).is_some() {
                                panic!("inserting into hll rep must yield hll rep");
                            };
                        }
                        *self = Sketch::Hll(hll);
                    }
                    Sketch::Array(lhs_arr) => {
                        let mut hll = rhs_hll.clone();
                        for &h in &**lhs_arr {
                            // todo: gross don't use deref
                            if hll.insert_encoded_hash(h).is_some() {
                                panic!("inserting into hll rep must yield hll rep");
                            };
                        }
                        *self = Sketch::Hll(hll);
                    }
                    Sketch::Hll(lhs_hll) => {
                        lhs_hll.merge(rhs_hll);
                    }
                }
            }
        }
    }
}

impl<const P: usize, const W: usize> Default for Sketch<P, W> {
    fn default() -> Self {
        Sketch::Small(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use wyhash::WyHash;

    #[cfg(feature = "with_digest")]
    #[test]
    fn test_estimator_with_prefix() {
        use sha2::Sha256;

        let mut estimator1: Sketch = Sketch::default();
        assert_eq!(estimator1.estimate(), 0);
        estimator1.insert(Element::from_digest_with_prefix::<Sha256>("secret", "hello"));
        assert_eq!(estimator1.estimate(), 1);

        let mut estimator2: Sketch = Sketch::default();
        estimator2.insert(Element::from_digest_with_prefix::<Sha256>("secret", "hello"));
        estimator1.merge(&estimator2);
        assert_eq!(estimator1.estimate(), 1);

        let mut estimator3: Sketch = Sketch::default();
        estimator3.insert(Element::from_digest_with_prefix::<Sha256>("sauce", "hello"));
        estimator1.merge(&estimator3);
        assert_eq!(estimator1.estimate(), 2);
    }

    #[test]
    fn small_size() {
        assert_eq!(Sketch::<0, 0>::default().size_of(), 8);
    }

    #[test_case(0 => "representation: Small(estimate: 0), avg_err: 0.0000")]
    #[test_case(1 => "representation: Small(estimate: 1), avg_err: 0.0000")]
    #[test_case(2 => "representation: Small(estimate: 2), avg_err: 0.0000")]
    #[test_case(3 => "representation: Array(estimate: 3), avg_err: 0.0000")]
    #[test_case(4 => "representation: Array(estimate: 4), avg_err: 0.0000")]
    #[test_case(8 => "representation: Array(estimate: 8), avg_err: 0.0000")]
    #[test_case(16 => "representation: Array(estimate: 16), avg_err: 0.0000")]
    #[test_case(17 => "representation: Array(estimate: 17), avg_err: 0.0000")]
    #[test_case(28 => "representation: Array(estimate: 28), avg_err: 0.0000")]
    #[test_case(29 => "representation: Array(estimate: 29), avg_err: 0.0000")]
    #[test_case(56 => "representation: Array(estimate: 56), avg_err: 0.0000")]
    #[test_case(57 => "representation: Array(estimate: 57), avg_err: 0.0000")]
    #[test_case(128 => "representation: Array(estimate: 128), avg_err: 0.0000")]
    #[test_case(129 => "representation: Hll(estimate: 131), avg_err: 0.0001")]
    #[test_case(256 => "representation: Hll(estimate: 264), avg_err: 0.0119")]
    #[test_case(512 => "representation: Hll(estimate: 512), avg_err: 0.0151")]
    #[test_case(1024 => "representation: Hll(estimate: 1033), avg_err: 0.0172")]
    #[test_case(10_000 => "representation: Hll(estimate: 10417), avg_err: 0.0281")]
    #[test_case(100_000 => "representation: Hll(estimate: 93099), avg_err: 0.0351")]
    fn test_estimator_p10_w5(n: usize) -> String {
        evaluate_sketch(
            Sketch::<10, 5>::default(),
            n,
        )
    }

    #[test_case(0 => "representation: Small(estimate: 0), avg_err: 0.0000")]
    #[test_case(1 => "representation: Small(estimate: 1), avg_err: 0.0000")]
    #[test_case(2 => "representation: Small(estimate: 2), avg_err: 0.0000")]
    #[test_case(3 => "representation: Array(estimate: 3), avg_err: 0.0000")]
    #[test_case(4 => "representation: Array(estimate: 4), avg_err: 0.0000")]
    #[test_case(8 => "representation: Array(estimate: 8), avg_err: 0.0000")]
    #[test_case(16 => "representation: Array(estimate: 16), avg_err: 0.0000")]
    #[test_case(32 => "representation: Array(estimate: 32), avg_err: 0.0000")]
    #[test_case(64 => "representation: Array(estimate: 64), avg_err: 0.0000")]
    #[test_case(128 => "representation: Array(estimate: 128), avg_err: 0.0000")]
    #[test_case(129 => "representation: Hll(estimate: 130), avg_err: 0.0001")]
    #[test_case(256 => "representation: Hll(estimate: 254), avg_err: 0.0029")]
    #[test_case(512 => "representation: Hll(estimate: 498), avg_err: 0.0068")]
    #[test_case(1024 => "representation: Hll(estimate: 1012), avg_err: 0.0130")]
    #[test_case(4096 => "representation: Hll(estimate: 4105), avg_err: 0.0089")]
    #[test_case(10_000 => "representation: Hll(estimate: 10068), avg_err: 0.0087")]
    #[test_case(100_000 => "representation: Hll(estimate: 95628), avg_err: 0.0182")]
    fn test_estimator_p12_w6(n: usize) -> String {
        evaluate_sketch(
            Sketch::<12, 6>::default(),
            n,
        )
    }

    #[test_case(0 => "representation: Small(estimate: 0), avg_err: 0.0000")]
    #[test_case(1 => "representation: Small(estimate: 1), avg_err: 0.0000")]
    #[test_case(2 => "representation: Small(estimate: 2), avg_err: 0.0000")]
    #[test_case(3 => "representation: Array(estimate: 3), avg_err: 0.0000")]
    #[test_case(4 => "representation: Array(estimate: 4), avg_err: 0.0000")]
    #[test_case(8 => "representation: Array(estimate: 8), avg_err: 0.0000")]
    #[test_case(16 => "representation: Array(estimate: 16), avg_err: 0.0000")]
    #[test_case(32 => "representation: Array(estimate: 32), avg_err: 0.0000")]
    #[test_case(64 => "representation: Array(estimate: 64), avg_err: 0.0000")]
    #[test_case(128 => "representation: Array(estimate: 128), avg_err: 0.0000")]
    #[test_case(129 => "representation: Hll(estimate: 129), avg_err: 0.0000")]
    #[test_case(256 => "representation: Hll(estimate: 256), avg_err: 0.0000")]
    #[test_case(512 => "representation: Hll(estimate: 511), avg_err: 0.0004")]
    #[test_case(1024 => "representation: Hll(estimate: 1022), avg_err: 0.0014")]
    #[test_case(4096 => "representation: Hll(estimate: 4100), avg_err: 0.0009")]
    #[test_case(10_000 => "representation: Hll(estimate: 10007), avg_err: 0.0008")]
    #[test_case(100_000 => "representation: Hll(estimate: 100240), avg_err: 0.0011")]
    fn test_estimator_p18_w6(n: usize) -> String {
        evaluate_sketch(
            Sketch::<18, 6>::default(),
            n,
        )
    }

    fn evaluate_sketch<const P: usize, const W: usize>(
        mut e: Sketch<P, W>,
        n: usize,
    ) -> String {
        let mut total_relative_error: f64 = 0.0;
        for i in 0..n {
            e.insert(Element::from_hasher_default::<WyHash>(&i));
            let estimate = e.estimate() as f64;
            let actual = (i + 1) as f64;
            let error = estimate - actual;
            let relative_error = error.abs() / actual;
            total_relative_error += relative_error;
        }

        let avg_relative_error = total_relative_error / ((n + 1) as f64);

        // Compute the expected standard error for HyperLogLog based on the precision
        let standard_error = 1.04 / 2.0f64.powi(P as i32).sqrt();
        let tolerance = 1.2;

        assert!(
            avg_relative_error <= standard_error * tolerance,
            "Average relative error {} exceeds acceptable threshold {}",
            avg_relative_error,
            standard_error * tolerance
        );

        format!(
            "representation: {:?}, avg_err: {:.4}",
            e, avg_relative_error
        )
    }

    #[test_case(0, 0 => "Small(estimate: 0)")]
    #[test_case(0, 1 => "Small(estimate: 1)")]
    #[test_case(1, 0 => "Small(estimate: 1)")]
    #[test_case(1, 1 => "Small(estimate: 2)")]
    #[test_case(1, 2 => "Array(estimate: 3)")]
    #[test_case(2, 1 => "Array(estimate: 3)")]
    #[test_case(2, 2 => "Array(estimate: 4)")]
    #[test_case(2, 3 => "Array(estimate: 5)")]
    #[test_case(2, 4 => "Array(estimate: 6)")]
    #[test_case(4, 2 => "Array(estimate: 6)")]
    #[test_case(3, 2 => "Array(estimate: 5)")]
    #[test_case(3, 3 => "Array(estimate: 6)")]
    #[test_case(3, 4 => "Array(estimate: 7)")]
    #[test_case(4, 3 => "Array(estimate: 7)")]
    #[test_case(4, 4 => "Array(estimate: 8)")]
    #[test_case(4, 8 => "Array(estimate: 12)")]
    #[test_case(8, 4 => "Array(estimate: 12)")]
    #[test_case(4, 12 => "Array(estimate: 16)")]
    #[test_case(12, 4 => "Array(estimate: 16)")]
    #[test_case(1, 127 => "Array(estimate: 128)")]
    #[test_case(1, 128 => "Hll(estimate: 130)")]
    #[test_case(127, 1 => "Array(estimate: 128)")]
    #[test_case(128, 1 => "Hll(estimate: 130)")]
    #[test_case(128, 128 => "Hll(estimate: 254)")]
    #[test_case(512, 512 => "Hll(estimate: 1012)")]
    #[test_case(10000, 0 => "Hll(estimate: 10068)")]
    #[test_case(0, 10000 => "Hll(estimate: 10068)")]
    #[test_case(4, 10000 => "Hll(estimate: 10068)")]
    #[test_case(10000, 4 => "Hll(estimate: 10068)")]
    #[test_case(17, 10000 => "Hll(estimate: 10073)")]
    #[test_case(10000, 17 => "Hll(estimate: 10073)")]
    #[test_case(10000, 10000 => "Hll(estimate: 19974)")]
    fn test_merge(lhs_n: usize, rhs_n: usize) -> String {
        let mut lhs = Sketch::<12, 6>::default();
        for i in 0..lhs_n {
            lhs.insert(Element::from_hasher_default::<WyHash>(i));
        }

        let mut rhs = Sketch::<12, 6>::default();
        for i in lhs_n..lhs_n + rhs_n {
            rhs.insert(Element::from_hasher_default::<WyHash>(i));
        }

        lhs.merge(&rhs);

        format!("{:?}", lhs)
    }

    #[test]
    fn test_insert() {
        // Create a new CardinalityEstimator.
        let mut e = Sketch::<12, 6>::default();

        // Ensure initial estimate is 0.
        assert_eq!(e.estimate(), 0);

        // Insert a test item and validate estimate.
        e.insert(Element::from_hasher_default::<WyHash>("test item 1"));
        assert_eq!(e.estimate(), 1);

        // Re-insert the same item, estimate should remain the same.
        e.insert(Element::from_hasher_default::<WyHash>("test item 1"));
        assert_eq!(e.estimate(), 1);

        // Insert a new distinct item, estimate should increase.
        e.insert(Element::from_hasher_default::<WyHash>("test item 2"));
        assert_eq!(e.estimate(), 2);
    }

}
