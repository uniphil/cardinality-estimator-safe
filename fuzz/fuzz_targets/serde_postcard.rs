#![no_main]

use cardinality_estimator_safe::{CardinalityEstimator, Representation};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(rep) = postcard::from_bytes::<Representation>(data) {
        let mut estimator: CardinalityEstimator<usize> = rep.into();
        estimator.insert(&1);
        assert!(estimator.estimate() > 0);
    }
});
