use std::hash::{Hash, Hasher};

/// Associative `BuildingBlock` wrapper with multiple sets/buckets.
///
/// This [`BuildingBlock`](trait.BuildingBlock.html) is implemented as an
/// array of buckets where each bucket is itself a
/// [`BuildingBlock`](trait.BuildingBlock.html). We may use both a "set" or
/// a "bucket" to qualify the latter. The primary goal of this container is
/// to balance the access to multiple containers while offering some
/// amount of parallelism when using it concurrently.
///
/// Keys inserted in this container must be hashable and their hash value
/// is used to associate them with a specific bucket. Association between
/// keys and buckets cannot be changed once this container is instantiated.
/// Below is how keys and buckets is associated. This can be used to provide
/// a custom `Hasher` that will tune how keys are grouped into buckets.
///
/// ```
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::{Hash, Hasher};
///
/// // Something to hash keys.
/// let mut hasher = DefaultHasher::new();
/// // Let's suppose we have 10 buckets.
/// let n_buckets = 10;
/// // The key for which we want to find the destination bucket.
/// let key = "some key";
/// // The key hash value.
/// key.hash(&mut hasher);
/// let key_hash = hasher.finish();
/// // This is the bucket where this key is assigned.
/// let key_bucket_index = key_hash % n_buckets;
/// ```
///
/// Since a key can only go in one bucket, the container may refuse
/// insertions before it is actually full if the target bucket is full.
///
/// [`Get`](trait.Get.html) and [`Concurrent`](trait.Concurrent.html)
/// traits are inherited from the type of container used to build this
/// associative container. If the buckets bear any of these traits,
/// then so does [`Associative`] container.
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
