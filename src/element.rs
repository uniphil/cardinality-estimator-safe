#[cfg(feature = "with_digest")]
use digest::Digest;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

/// A member that can be inserted into a Sketch
///
/// - You **must** use the same hash configuration for **all** elements inserted
///   into a sketch
/// - You **should not** insert elements of different types to the same sketch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Element<const P: usize = 12, const W: usize = 6>(pub(crate) u32);

impl<const P: usize, const W: usize> Element<P, W> {
    /// Wrap an already-hashed element for insertion
    ///
    /// For advanced use cases: if your input value is not `std::hash::Hash` or
    /// you want to use some other hash function not supported by the other
    /// initialisers, you can hash the elements yourself and use this.
    ///
    /// Note that you can almost never *skip* hashing: the estimator relies on
    /// random-like distribution of bits in the element's hash to work.
    #[inline]
    pub fn from_hashed(hashed: u64) -> Self {
        // Ensure that `P` and `W` are in correct range at compile time
        const { assert!(P >= 4 && P <= 18 && W >= 4 && W <= 6) }
        let idx = (hashed as u32) & ((1 << (32 - W - 1)) - 1);
        let rank = (!hashed >> P).trailing_zeros() + 1;
        Self((idx << W) | rank)
    }

    /// Wrap a `Hash` element with a `BuildHasher` instance
    ///
    /// The `BuildHasher` can initialize state for secret/salting, but if you
    /// need that, consider enabling the `use_digest` feature and using a secure
    /// hash with `from_digest_with_prefix`.
    #[inline]
    pub fn from_hasher(element: impl Hash, hasher: impl BuildHasher) -> Self {
        Self::from_hashed(hasher.hash_one(&element))
    }

    /// Wrap a `Hash` element with a `Hasher` specified by type
    #[inline]
    pub fn from_hasher_default<H: Hasher + Default>(element: impl Hash) -> Self {
        Self::from_hasher(element, BuildHasherDefault::<H>::default())
    }

    /// Wrap element bytes with a secret prefix hashed by any `Digest` hasher
    ///
    /// This can help resist offline attacks against your estimates if a user
    /// can influence the content of inserted elements.
    ///
    /// The secret prefix must be fixed and cannot be rotated for a sketch. If
    /// it changes, future estimates will be invalidated by any inserts and
    /// merges.
    #[cfg(feature = "with_digest")]
    #[inline]
    pub fn from_digest_with_prefix<D: Digest>(
        prefix: impl AsRef<[u8]>,
        element: impl AsRef<[u8]>,
    ) -> Self {
        let mut hasher = D::new_with_prefix(prefix);
        hasher.update(element);
        let first8: [u8; 8] = hasher // TODO: there's def a better way to split the first 8 from GenericArray with type checking
            .finalize()
            .as_slice()
            .get(0..8)
            .expect("digest output must be at least 8 bytes")
            .try_into()
            .unwrap();
        Self::from_hashed(u64::from_le_bytes(first8))
    }

    /// Wrap element bytes with a hashed by any `Digest` hasher
    #[cfg(feature = "with_digest")]
    #[inline]
    pub fn from_digest_oneshot<D: Digest>(element: impl AsRef<[u8]>) -> Self {
        let first8: [u8; 8] = D::digest(element) // TODO: there's def a better way to split the first 8 from GenericArray with type checking
            .as_slice()
            .get(0..8)
            .expect("digest output must be at least 8 bytes")
            .try_into()
            .unwrap();
        Self::from_hashed(u64::from_le_bytes(first8))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use wyhash::WyHash;

    #[test]
    fn test_blah() {
        let _: Element = Element::from_hasher_default::<WyHash>(&123);
    }

    #[cfg(feature = "with_digest")]
    #[test]
    fn test_bleh() {
        use sha2::Sha256;
        let _: Element = Element::from_digest_oneshot::<Sha256>(&[123]);
    }
}
