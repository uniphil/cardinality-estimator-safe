#[cfg(feature = "with_serde")]
fn main() {
    let mut estimator =
        cardinality_estimator::CardinalityEstimator::<usize, wyhash::WyHash, 8, 5>::new();

    println!(
        "serialized empty estimator (small): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    estimator.insert(&0);

    println!(
        "serialized with one insert (small): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    estimator.insert(&1);
    estimator.insert(&2);

    println!(
        "serialized with three inserts (array): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    for i in 3..1000 {
        estimator.insert(&i);
    }

    println!(
        "serialized with many inserts (HLL): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );
}

#[cfg(not(feature = "with_serde"))]
fn main() -> Result<(), u32> {
    eprintln!("this example requires --features with_serde");
    Err(1)
}
