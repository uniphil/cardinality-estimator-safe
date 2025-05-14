//! # Serde module for CardinalityEstimator

use crate::hyperloglog::HyperLogLog;
use serde::de::{self, SeqAccess, Visitor};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use std::fmt;

impl<const P: usize, const W: usize> Serialize for HyperLogLog<P, W> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        assert_eq!(Self::HLL_SLICE_LEN, self.registers.len());
        let mut tup = serializer.serialize_seq(Some(Self::HLL_SLICE_LEN))?;
        for r in &self.registers {
            tup.serialize_element(r)?;
        }
        tup.end()
    }
}

struct TupleU32Visitor(usize);

impl<'de> Visitor<'de> for TupleU32Visitor {
    type Value = Vec<u32>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple of u32s")
    }

    fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self(expected_len) = self;
        let mut registers: Self::Value = Vec::with_capacity(expected_len);
        for i in 0..expected_len {
            let el = access.next_element()?.ok_or_else(|| {
                de::Error::custom(format!(
                    "could not find register at index {i} (of {expected_len} expected)"
                ))
            })?;
            registers.push(el);
        }
        Ok(registers)
    }
}

impl<'de, const P: usize, const W: usize> Deserialize<'de> for HyperLogLog<P, W> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = deserializer.deserialize_seq(TupleU32Visitor(Self::HLL_SLICE_LEN))?;
        Ok(HyperLogLog::from_registers(registers))
    }
}

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
