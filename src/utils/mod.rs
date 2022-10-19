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
