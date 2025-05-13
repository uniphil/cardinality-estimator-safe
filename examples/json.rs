#[cfg(feature = "with_serde")]
fn main() {
    use cardinality_estimator_safe::CardinalityEstimator;
    use std::hash::BuildHasherDefault;
    use wyhash::WyHash;
    let mut estimator: CardinalityEstimator<usize, BuildHasherDefault<WyHash>, 8, 5> =
        cardinality_estimator_safe::CardinalityEstimator::new();

    println!(
        "serialized empty estimator (small): {}",
        serde_json::to_string_pretty(estimator.representation()).unwrap()
    );

    estimator.insert(&0);

    println!(
        "serialized with one insert (small): {}",
        serde_json::to_string_pretty(estimator.representation()).unwrap()
    );

    estimator.insert(&1);
    estimator.insert(&2);

    println!(
        "serialized with three inserts (array): {}",
        serde_json::to_string_pretty(estimator.representation()).unwrap()
    );

    for i in 3..1000 {
        estimator.insert(&i);
    }

    println!(
        "serialized with many inserts (HLL): {}",
        serde_json::to_string_pretty(estimator.representation()).unwrap()
    );
}

#[cfg(not(feature = "with_serde"))]
fn main() -> Result<(), u32> {
    eprintln!("this example requires --features with_serde");
    Err(1)
}
