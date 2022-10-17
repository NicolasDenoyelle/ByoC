use std::hash::{Hash, Hasher};

/// Associative `BuildingBlock` wrapper with multiple sets/buckets.
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// This building block is implemented as an array of building blocks.
/// Keys inserted in this container must be hashable to find in which bucket
/// it should be stored/retrieved.
///
/// Since a key can only go in one bucket, the container may refuse
/// insertions before it is actually full if one of the target buckets is full.
///
/// When [popping](trait.BuildingBlock.html#tymethod.pop) elements,
/// the policy is to balance buckets element count rather than strictly
/// pop values in descending order. This is because popping values in descending
/// order requires lot of [`pop()`](trait.BuildingBlock.html#tymethod.pop)
/// and [`push()`](trait.BuildingBlock.html#tymethod.push) operations whereas
/// balancing buckets can be done by looking at the count of each bucket once
/// and [popping](trait.BuildingBlock.html#tymethod.pop) once per bucket.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// [`Get`](trait.Get.html) and [`Concurrent`](trait.Concurrent.html)
/// traits are inherited from the type of container used to build this
/// associative container.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Associative};
/// use std::collections::hash_map::DefaultHasher;
///
/// // Build a Array cache of 2 sets. Each set hold one element.
/// let mut c = Associative::new(vec![Array::new(2), Array::new(2)],
///                              DefaultHasher::new());
///
/// // BuildingBlock as room for first and second element and returns None.
/// assert!(c.push(vec![(0, 4)]).pop().is_none());
/// assert!(c.push(vec![(1, 12)]).pop().is_none());
///
/// // Then we don't know if a set is full. Next insertion may pop:
/// match c.push(vec![(2, 14)]).pop() {
///       None => { println!("Still room for one more"); }
///       Some((key, value)) => {
///             assert!(key == 1);
///             assert!(value == 12);
///       }
/// }
///```
///
/// [`Associative`] can also be built from a
/// [builder pattern](builder/builders/struct.AssociativeBuilder.html) and a
/// [configuration](config/struct.AssociativeConfig.html).
pub struct Associative<C, H: Hasher + Clone> {
    pub(super) containers: Vec<C>,
    pub(super) hasher: H,
}

impl<C, H: Hasher + Clone> Associative<C, H> {
    /// Construct a new associative container.
    ///
    /// This function builds an associative container using other
    /// containers as sets.
    pub fn new(sets: Vec<C>, key_hasher: H) -> Self {
        Associative {
            containers: sets,
            hasher: key_hasher,
        }
    }

    pub(super) fn set<K: Hash>(&self, key: K) -> usize {
        let n_sets = self.containers.len();
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let i = hasher.finish();
        (i % (n_sets as u64)) as usize
    }
}
