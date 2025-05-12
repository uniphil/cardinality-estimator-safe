#![no_main]

use arbitrary::Arbitrary;
use cardinality_estimator::estimator::CardinalityEstimator;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Hash, PartialEq, Debug)]
struct Datum(usize);

#[derive(Arbitrary, Debug)]
struct Data(Vec<Datum>);

fuzz_target!(|data: Data| {
    let mut estimator = CardinalityEstimator::<Datum>::new();
    for d in &data.0 {
        estimator.insert(&d);
    }
    let before_estimate = estimator.estimate();

    let serialized = postcard::to_allocvec(&estimator).unwrap();
    let mut roundtripped = postcard::from_bytes::<CardinalityEstimator<Datum>>(&serialized).unwrap();
    assert_eq!(before_estimate, roundtripped.estimate());

    roundtripped.insert(&Datum(1));

    let before_estimate = roundtripped.estimate();
    let mut serialized = serde_json::to_string(&roundtripped).unwrap();
    serialized += " ";
    let roundtripped2: CardinalityEstimator<Datum> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(before_estimate, roundtripped2.estimate());
});
