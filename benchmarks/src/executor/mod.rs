//! Utilities for exercising an initialized container.

pub trait Executor {
    fn run(&mut self) -> crate::utils::csv::Record;
}

mod random;
pub use random::RandomActionExecutor;
