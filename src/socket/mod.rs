mod client;
pub use client::SocketClient;
mod building_block;

mod server;
pub use server::{ServerLoopEvent, SocketServer};

mod server_thread;
pub use server_thread::{ServerThreadBuilder, ServerThreadHandle};

pub mod builder;

#[cfg(feature = "config")]
pub mod config;

mod message;

mod error;

#[cfg(test)]
pub mod tests;
