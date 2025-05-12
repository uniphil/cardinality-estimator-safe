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
    let serialized = postcard::to_allocvec(&estimator).unwrap();
    let mut roundtripped: CardinalityEstimator<Datum> = postcard::from_bytes(&serialized).unwrap();
    roundtripped.insert(&Datum(1));
});
