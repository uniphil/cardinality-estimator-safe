use cardinality_estimator_safe::{Sketch, Element};
use wyhash::WyHash;

fn main() {
    let mut estimator1: Sketch = Sketch::default();
    for i in 0..10 {
        estimator1.insert(Element::from_hasher_default::<WyHash>(&i));
    }
    println!("estimator1 estimate = {}", estimator1.estimate());

    let mut estimator2 = Sketch::default();
    for i in 10..15 {
        estimator2.insert(Element::from_hasher_default::<WyHash>(&i));
    }
    println!("estimator2 estimate = {}", estimator2.estimate());

    estimator1.merge(&estimator2);
    println!("merged estimate = {}", estimator1.estimate());
}
