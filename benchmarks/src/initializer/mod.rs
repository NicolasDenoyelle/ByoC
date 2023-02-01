//! Utilities for initializing a container state.

pub trait Initializer<B> {
    fn initialize(self, initializee: &mut B);
}

mod push;
pub use push::PushInitializer;
