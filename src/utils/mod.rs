/// [`Stream`](../../struct.Stream.html) struct helpers.
#[cfg(feature = "stream")]
pub mod stream {
    pub use crate::stream::{
        FileStream, Stream, StreamBase, StreamFactory,
        TempFileStreamFactory, VecStream, VecStreamFactory,
    };
}

/// [`Profiler`](../struct.Profiler.html) struct helpers.
pub mod profiler {
    pub use crate::profiler::ProfilerOutputKind;
}

/// [`Associative`](../struct.Associative.html) struct helpers.
pub mod associative {
    pub use crate::associative::ExclusiveHasher;
}

mod lifetime;

/// Objects returned by [`Get`](../../trait.Get.html) and
/// [`GetMut`](../../trait.GetMut.html) traits implementations.
pub mod get {
    pub use super::lifetime::LifeTimeGuard;
}
