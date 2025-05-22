use std::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault};
#[cfg(feature = "with_digest")]
use digest::Digest;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Element<const P: usize = 12, const W: usize = 6>(pub(crate) u32);

impl<const P: usize, const W: usize> Element<P, W> {
    #[inline]
    pub fn from_hashed(hashed: u64) -> Self {
        // Ensure that `P` and `W` are in correct range at compile time
        const { assert!(P >= 4 && P <= 18 && W >= 4 && W <= 6) }
        let idx = (hashed as u32) & ((1 << (32 - W - 1)) - 1);
        let rank = (!hashed >> P).trailing_zeros() + 1;
        Self((idx << W) | rank)
    }

    #[inline]
    pub fn from_hasher(element: impl Hash, hasher: impl BuildHasher) -> Self {
        Self::from_hashed(hasher.hash_one(&element))

    }

    #[inline]
    pub fn from_hasher_default<H: Hasher + Default>(element: impl Hash) -> Self {
        Self::from_hasher(element, BuildHasherDefault::<H>::default())
    }

    #[cfg(feature = "with_digest")]
    #[inline]
    pub fn from_digest_with_prefix<D: Digest>(
        prefix: impl AsRef<[u8]>,
        element: impl AsRef<[u8]>,
    ) -> Self {
        let mut hasher = D::new_with_prefix(prefix);
        hasher.update(element);
        let first8: [u8; 8] = hasher.finalize() // TODO: there's def a better way to split the first 8 from GenericArray with type checking
            .as_slice()
            .get(0..8)
            .expect("digest output must be at least 8 bytes")
            .try_into()
            .unwrap();
        Self::from_hashed(u64::from_le_bytes(first8.into()))
    }

    #[cfg(feature = "with_digest")]
    #[inline]
    pub fn from_digest_oneshot<D: Digest>(element: impl AsRef<[u8]>) -> Self {
        let first8: [u8; 8] = D::digest(element) // TODO: there's def a better way to split the first 8 from GenericArray with type checking
            .as_slice()
            .get(0..8)
            .expect("digest output must be at least 8 bytes")
            .try_into()
            .unwrap();
        Self::from_hashed(u64::from_le_bytes(first8.into()))
    }
}


#[cfg(test)]
pub mod tests {
    use wyhash::WyHash;
    use super::*;

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
