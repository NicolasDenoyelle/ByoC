use crate::internal::math::log2;
use std::hash::Hasher;

/// A single hasher for multiple stages of associative containers.
///
/// The purpose of this structure is to concatenate multiple layer of
/// [`Associative`](struct.Associative.html) building block with a
/// single hasher but different hashes on each level.
///
/// This is needed when building multiple stages of associative containers
/// with the same hasher, to avoid ending up using only one bucket in every
/// level but the first one, thus defeating the purpose of the associative
/// container.
///
/// This hasher can be used to create another `ExclusiveHasher` returning a
/// disjoint set of bits compared to the first one from the same hash bits.
/// See [`next()`](struct.ExclusiveHasher.html#tymethod.next) method.
///
/// This structure is used by the [`Associative`](../../struct.Associative.html)
/// container
/// [builder](../../associative/builder/struct.AssociativeBuilder.html) to stack
/// multiple associative containers.
pub struct ExclusiveHasher<H: Hasher> {
    hasher: H,
    mask: u64,
    rshift: u8,
    nbits: u8,
}

impl<H: Hasher> Hasher for ExclusiveHasher<H> {
    fn finish(&self) -> u64 {
        let h = self.hasher.finish();
        (h & self.mask) >> self.rshift
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }
}

impl<H: Hasher + Clone> Clone for ExclusiveHasher<H> {
    fn clone(&self) -> Self {
        ExclusiveHasher {
            hasher: self.hasher.clone(),
            mask: self.mask,
            rshift: self.rshift,
            nbits: self.nbits,
        }
    }
}

impl<H: Hasher + Clone> ExclusiveHasher<H> {
    /// Create a new multiset hasher from another hasher
    /// that returns hash with a maximum value of at least nsets.
    pub fn new(hasher: H, nsets: usize) -> Self {
        let nbits = log2(nsets as u64) + 1;
        ExclusiveHasher {
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
        Ok(ExclusiveHasher {
            hasher: self.hasher.clone(),
            mask,
            rshift,
            nbits,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ExclusiveHasher;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn multiset_hasher() {
        let mut h = ExclusiveHasher::new(DefaultHasher::new(), 1);
        for i in 1..64 {
            h = h.next(1).unwrap();
            assert_eq!(h.nbits, 1);
            assert_eq!(h.rshift, i);
            assert_eq!(h.mask >> h.rshift, 1);
        }
        assert!(h.next(1).is_err());

        let mut h = ExclusiveHasher::new(DefaultHasher::new(), 7);
        for i in 1..21 {
            h = h.next(7).unwrap();
            assert_eq!(h.nbits, 3);
            assert_eq!(h.rshift, i * 3);
            assert_eq!(h.mask >> h.rshift, 7);
        }
        assert!(h.next(7).is_err());
    }
}
