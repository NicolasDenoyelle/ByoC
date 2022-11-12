/// `Stream` struct helpers.
#[cfg(feature = "stream")]
pub mod stream {
    pub use crate::stream::{
        FileStream, Stream, StreamBase, StreamFactory,
        TempFileStreamFactory, VecStream, VecStreamFactory,
    };
}

/// `Profiler` struct helpers.
pub mod profiler {
    pub use crate::profiler::ProfilerOutputKind;
}

/// `Associative` struct helpers.
pub mod associative {
    pub use crate::associative::ExclusiveHasher;
}

mod lifetime;

/// Objects returned by `Get` and `GetMut` traits implementations.
pub mod get {
    pub use super::lifetime::LifeTimeGuard;
}

#[cfg(feature = "socket")]
/// Utils to spawn a thread running a `SocketServer`.
pub mod socket {
    pub use crate::socket::{ServerThreadBuilder, ServerThreadHandle};
}

/// Eviction policies and related traits and modules.
///
/// Containers eviction is implemented by the
/// [`building blocks`](../trait.BuildingBlock.html#implementors)
/// themselves when calling the
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop) method.
/// The when sthe container implements the
/// [`Ordered`](trait.Ordered.html) trait, the pop
/// method will try to take out the elements with the highest value.
///
/// The [`Policy`](../struct.Policy.html) container is wrapper around
/// such a container (although the container does not need to carry the
/// [`Ordered`](trait.Ordered.html) trait bound) that will wrap values into
/// a [`Reference`](trait.Reference.html)
/// cell ordering values in the container with a specific policy.
///
/// This is a generic, but potentially inefficient, way to customize the
/// eviction policy on a wide range of containers.
///
/// [`Lrfu`](struct.Lrfu.html) and [`Lru`](struct.Lru.html) policies will
/// change the order of elements
/// in the container when they are accessed from within the container
/// using [`Get`](../trait.Get.html) and [`GetMut`](../trait.GetMut.html)
/// traits. This is potentially dangerous! Indeed, if the container relies
/// on the order of its elements (for instance it uses a
/// [`std::collections::BTreeSet`]), then
/// accessing elements inside the container will make things dicey.
/// If the container does not
/// implement the [`Ordered`](trait.Ordered.html) trait bound, it is probably
/// a bad idea to use
/// on of these policies.
///
/// ### Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Policy};
/// use byoc::utils::policy::Fifo;
///
/// let mut c = Policy::new(Array::new(3), Fifo::new());
/// assert_eq!(c.push(vec![("item1",1u16), ("item2",2u16), ("item0",0u16)])
///             .len(), 0);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
/// ```
pub mod policy {
    pub use crate::policy::{
        timestamp, Fifo, Lrfu, Lru, Ordered, Reference, ReferenceFactory,
    };
}
