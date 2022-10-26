mod client;
pub use client::SocketClient;
pub mod builder;
mod building_block;
#[cfg(feature = "config")]
pub mod config;
mod get;

mod server;
pub use server::{ServerLoopEvent, SocketServer};

mod message;
pub(crate) mod server_thread;

mod error;

#[cfg(test)]
pub mod tests;
