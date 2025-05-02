#![no_main]

use cardinality_estimator_safe::estimator::CardinalityEstimator;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(mut estimator) = serde_json::from_slice::<CardinalityEstimator<usize>>(data) {
        estimator.insert(&1);
        assert!(estimator.estimate() > 0);
    }
});
