#![no_main]

use serde_json::json;
use cardinality_estimator_safe::{CardinalityEstimator, Representation};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // pretty naiive version

    let v = match data.get(0) {
        Some(n) if n % 3 == 0 => {
            let mut rest: i64 = 0;
            for d in data.get(1..).unwrap_or(&[]) {
                rest <<= 8;
                rest += *d as i64;
            }
            json!({"s": rest}) // small representation
        }
        Some(n) if n % 3 == 1 => {
            json!({"a": data.get(1..).unwrap_or(&[])}) // array rep
        }
        Some(_) => {
            json!({"h": data.get(1..).unwrap_or(&[])}) // hyperloglog rep
        }
        None => json!({})
    };

    if let Ok(rep) = serde_json::from_value::<Representation>(v) {
        let mut estimator: CardinalityEstimator<usize> = rep.into();
        estimator.insert(&1);
        assert!(estimator.estimate() > 0);
    }
});
