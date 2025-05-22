//! # Serde module for CardinalityEstimator

use crate::array::{Array, MAX_CAPACITY as ARRAY_MAX_CAPACITY};
use crate::hyperloglog::HyperLogLog;
use serde::de::{self, SeqAccess, Visitor};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use std::fmt;

impl<const P: usize, const W: usize> Serialize for Array<P, W> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let els = &**self;
        let mut a = serializer.serialize_seq(Some(els.len()))?;
        for el in els {
            a.serialize_element(el)?;
        }
        a.end()
    }
}

impl<'de, const P: usize, const W: usize> Deserialize<'de> for Array<P, W> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let items: Vec<u32> = Deserialize::deserialize(deserializer)?;
        let found = items.len();
        if found < 3 {
            return Err(de::Error::invalid_length(
                found,
                &"array representation with at least 3 items",
            ));
        }
        if found > ARRAY_MAX_CAPACITY {
            return Err(de::Error::invalid_length(
                found,
                &format!("array representation with at most {ARRAY_MAX_CAPACITY} items").as_str(),
            ));
        }
        Ok(Array::from_items(items))
    }
}

/// Serialize the HyperLogLog representation
///
/// Serializing the zeros and harmonic_sum values is a choice that I'm rolling with
/// for now, because:
///
/// - it's cheap, 8 bytes per hll
/// - it *may* offer an optimized shortcut to extract estimates from serialized data
/// - it leaves flexibility to avoid recomputing, if the underlying serialized storage has sufficient integrity
///
/// the serialied data is sequence of u32s:
/// - 0: hll zeros
/// - 1: harmonic_sum (f32 transmuted to u32)
/// - 2..: registers array
impl<const P: usize, const W: usize> Serialize for HyperLogLog<P, W> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // shouldn't be necessary, but things have really gone wrong somewhere if not:
        assert_eq!(Self::HLL_SLICE_LEN, self.registers.len());

        let mut seq = serializer.serialize_seq(Some(Self::HLL_SLICE_LEN + 2))?;
        seq.serialize_element(&self.zeros)?;
        seq.serialize_element(&self.harmonic_sum.to_bits())?;

        for r in &self.registers {
            seq.serialize_element(r)?;
        }
        seq.end()
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
            let Some(el) = access.next_element()? else {
                return Err(de::Error::invalid_length(
                    i,
                    &format!("hyperloglog representation with length {expected_len}.").as_str(),
                ));
            };
            registers.push(el);
        }
        if let Some(remaining) = access.size_hint() {
            if remaining > 0 {
                return Err(de::Error::invalid_length(
                    expected_len,
                    &format!("hyperloglog representation with length {expected_len} (hint says {remaining} extra in sequence)").as_str(),
                ));
            }
        }
        Ok(registers)
    }
}

/// Deserialize the HyperLogLog representation
///
/// for now, the deserializer will: recompute the values from the sketch, and assert that
///
/// - the number of zeros must match exactly
/// - the harmonic sum must match within some error
///
/// otherwise there was likely some issue with the data in storage and it will be rejected.
/// for now, the *stored* harmonic_sum will ultimately be used, so that the deserialized
/// instance has the exact state of the pre-serialiezd one. it seems intuitively like this
/// might have slightly higher accumulated floating-point error though?
///
/// cheaper/less safe deserialization paths may be added in the future
impl<'de, const P: usize, const W: usize> Deserialize<'de> for HyperLogLog<P, W> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let stuff = deserializer.deserialize_seq(TupleU32Visitor(Self::HLL_SLICE_LEN + 2))?;
        let zeros = stuff[0];
        let harmonic_sum = f32::from_bits(stuff[1]);
        let registers = stuff.get(2..).unwrap().to_vec();

        assert_eq!(registers.len(), Self::HLL_SLICE_LEN);
        let mut hll = HyperLogLog::from_registers(registers);

        if hll.zeros != zeros {
            return Err(de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(zeros.into()),
                &format!(
                    "zeros to match the zeros from registers ({}) exactly",
                    hll.zeros
                )
                .as_str(),
            ));
        }

        // there is probably some nice math that could justify a minimal error threshold here
        // but i'm lazy and we just want to catch serialization issues which are going to be
        // really wildly wrong or not that important
        if (hll.harmonic_sum - harmonic_sum).abs() > 0.5 {
            return Err(de::Error::invalid_value(
                serde::de::Unexpected::Float(harmonic_sum.into()),
                &format!(
                    "harmonic_sum to match computed sum from registers ({}) closely",
                    hll.harmonic_sum
                )
                .as_str(),
            ));
        }

        hll.harmonic_sum = harmonic_sum;
        Ok(hll)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Sketch, Element};
    use test_case::test_case;
    use wyhash::WyHash;

    #[test_case(0; "empty set")]
    #[test_case(1; "single element")]
    #[test_case(2; "two distinct elements")]
    #[test_case(100; "hundred distinct elements")]
    #[test_case(10000; "ten thousand distinct elements")]
    fn test_serde(n: usize) {
        let mut original_estimator = Sketch::default();

        for i in 0..n {
            let item = &format!("item{}", i);
            original_estimator.insert(Element::from_hasher_default::<WyHash>(&item));
        }

        let serialized = serde_json::to_string(&original_estimator)
            .expect("serialization failed");
        assert!(
            !serialized.is_empty(),
            "serialized string should not be empty"
        );

        let deserialized_estimator: Sketch =
            serde_json::from_str::<Sketch>(&serialized)
                .expect("deserialization failed")
                .into();

        assert_eq!(
            original_estimator,
            deserialized_estimator
        );

        // run each case with postcard serialization as well

        let postcard_serialized = postcard::to_allocvec(&original_estimator)
            .expect("serialization failed");
        assert!(
            !postcard_serialized.is_empty(),
            "postcard_serialized bytes should not be empty"
        );

        let postcard_estimator: Sketch =
            postcard::from_bytes::<Sketch>(&postcard_serialized)
                .expect("deserialization failed")
                .into();

        assert_eq!(
            original_estimator,
            postcard_estimator
        );
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let invalid_json = "{ invalid_json_string }";
        let result: Result<Sketch, _> = serde_json::from_str(invalid_json);

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
        let result: Result<Sketch, _> = serde_json::from_slice(input);
        assert!(result.is_err());

        let result: Result<Sketch, _> = postcard::from_bytes(input);
        assert!(result.is_err());
    }
}
