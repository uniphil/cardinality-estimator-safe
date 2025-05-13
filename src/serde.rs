//! # Serde module for CardinalityEstimator
//!
//! This module now only provides basic tests for derived serializationa and deserialization.

#[cfg(test)]
pub mod tests {
    use crate::estimator::CardinalityEstimator;
    use crate::representation::Representation;
    use test_case::test_case;

    #[test_case(0; "empty set")]
    #[test_case(1; "single element")]
    #[test_case(2; "two distinct elements")]
    #[test_case(100; "hundred distinct elements")]
    #[test_case(10000; "ten thousand distinct elements")]
    fn test_serde(n: usize) {
        let mut original_estimator = CardinalityEstimator::<str>::new();

        for i in 0..n {
            let item = &format!("item{}", i);
            original_estimator.insert(&item);
        }

        let serialized = serde_json::to_string(original_estimator.representation())
            .expect("serialization failed");
        assert!(
            !serialized.is_empty(),
            "serialized string should not be empty"
        );

        let deserialized_estimator: CardinalityEstimator<str> =
            serde_json::from_str::<Representation>(&serialized)
                .expect("deserialization failed")
                .into();

        assert_eq!(
            original_estimator.representation(),
            deserialized_estimator.representation()
        );

        // run each case with postcard serialization as well

        let postcard_serialized = postcard::to_allocvec(original_estimator.representation())
            .expect("serialization failed");
        assert!(
            !postcard_serialized.is_empty(),
            "postcard_serialized bytes should not be empty"
        );

        let postcard_estimator: CardinalityEstimator<str> =
            postcard::from_bytes::<Representation>(&postcard_serialized)
                .expect("deserialization failed")
                .into();

        assert_eq!(
            original_estimator.representation(),
            postcard_estimator.representation()
        );
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let invalid_json = "{ invalid_json_string }";
        let result: Result<Representation, _> = serde_json::from_str(invalid_json);

        assert!(
            result.is_err(),
            "Deserialization should fail for invalid JSON"
        );
    }

    #[test_case("[12345,null]".as_bytes(); "case 1")]
    #[test_case(&[91, 49, 55, 44, 13, 10, 91, 13, 93, 93]; "case 2")]
    #[test_case(&[91, 51, 44, 10, 110, 117, 108, 108, 93, 122]; "case 3")]
    #[test_case(&[91, 51, 44, 10, 110, 117, 108, 108, 93]; "case 4")]
    fn test_failed_deserialization(input: &[u8]) {
        let result: Result<Representation, _> = serde_json::from_slice(input);
        assert!(result.is_err());

        let result: Result<Representation, _> = postcard::from_bytes(input);
        assert!(result.is_err());
    }
}
