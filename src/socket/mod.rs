mod client;
pub use client::SocketClient;
mod building_block;

mod server;
mod server_thread;
pub use server_thread::{ServerThreadBuilder, ServerThreadHandle};

pub mod builder;
#[cfg(feature = "config")]
pub mod config;

mod error;
mod message;
