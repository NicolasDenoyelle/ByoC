use crate::internal::bits::log2;
use std::hash::Hasher;

/// Hasher wrapper that returns a subset of the hash bits shifted to the
/// right.
///
/// The purpose of this structure is to concatenate multiple layer of
/// [`Associative`](struct.Associative.html) building block with one
/// different hasher but different hashes on each level.
/// See how to instantiate such a structure in
/// [builder examples](builder/associative/struct.AssociativeBuilder.html).
///
/// This hasher can be used to create another `MultisetHasher` returning a
/// disjoint set of bits compared to the first one from the hash bits.
/// See [`next()`](struct.MultisetHasher.html#tymethod.next) method.
pub struct MultisetHasher<H: Hasher> {
    hasher: H,
    mask: u64,
    rshift: u8,
    nbits: u8,
}

impl<H: Hasher> Hasher for MultisetHasher<H> {
    fn finish(&self) -> u64 {
        let h = self.hasher.finish();
        (h & self.mask) >> self.rshift
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }
}

impl<H: Hasher + Clone> Clone for MultisetHasher<H> {
    fn clone(&self) -> Self {
        MultisetHasher {
            hasher: self.hasher.clone(),
            mask: self.mask,
            rshift: self.rshift,
            nbits: self.nbits,
        }
    }
}

impl<H: Hasher + Clone> MultisetHasher<H> {
    /// Create a new multiset hasher from another hasher
    /// that returns hash with a maximum value of at least nsets.
    pub fn new(hasher: H, nsets: usize) -> Self {
        let nbits = log2(nsets as u64) + 1;
        MultisetHasher {
            hasher,
            mask: (!0u64) >> ((64u8 - nbits) as u64),
            rshift: 0u8,
            nbits,
        }
    }

    #[allow(clippy::result_unit_err)]
    /// Create a new multiset hasher that uses a disjoint set of bits
    /// of the hash value generated this hasher, located on the left
    /// of the set of bits used by this hasher.
    /// If there is not enough bits available to count up to `nsets`,
    /// an error is returned instead.
    pub fn next(&self, nsets: usize) -> Result<Self, ()> {
        let nbits = log2(nsets as u64) + 1;
        let rshift = self.nbits + self.rshift;

        if rshift + nbits > 64u8 {
            return Err(());
        }

        let mask = ((!0u64) >> ((64u8 - nbits) as u64)) << rshift;
        Ok(MultisetHasher {
            hasher: self.hasher.clone(),
            mask,
            rshift,
            nbits,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MultisetHasher;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn multiset_hasher() {
        let mut h = MultisetHasher::new(DefaultHasher::new(), 1);
        for i in 1..64 {
            h = h.next(1).unwrap();
            assert_eq!(h.nbits, 1);
            assert_eq!(h.rshift, i);
            assert_eq!(h.mask >> h.rshift, 1);
        }
        assert!(h.next(1).is_err());

        let mut h = MultisetHasher::new(DefaultHasher::new(), 7);
        for i in 1..21 {
            h = h.next(7).unwrap();
            assert_eq!(h.nbits, 3);
            assert_eq!(h.rshift, i * 3);
            assert_eq!(h.mask >> h.rshift, 7);
        }
        assert!(h.next(7).is_err());
    }
}
