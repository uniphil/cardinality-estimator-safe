#[cfg(feature = "with_serde")]
fn main() {
    use cardinality_estimator_safe::{Sketch, Element};

    use wyhash::WyHash;
    let mut estimator: Sketch<8, 5> = Sketch::default();

    println!(
        "serialized empty estimator (small): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    estimator.insert(Element::from_hasher_default::<WyHash>(&0));

    println!(
        "serialized with one insert (small): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    estimator.insert(Element::from_hasher_default::<WyHash>(&1));
    estimator.insert(Element::from_hasher_default::<WyHash>(&2));

    println!(
        "serialized with three inserts (array): {}",
        serde_json::to_string_pretty(&estimator).unwrap()
    );

    for i in 3..1000 {
        estimator.insert(Element::from_hasher_default::<WyHash>(&i));
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
