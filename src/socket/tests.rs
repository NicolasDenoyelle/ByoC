use crate::tests::{TestKey, TestValue};
use crate::utils::socket::{ServerThreadBuilder, ServerThreadHandle};
use crate::{Array, Sequential, SocketClient};
use std::time::Duration;

pub(super) fn make_server_client(
    capacity: usize,
    address: &str,
) -> (SocketClient, ServerThreadHandle) {
    let container =
        Sequential::new(Array::<(TestKey, TestValue)>::new(capacity));
    let server =
        ServerThreadBuilder::new(String::from(address), container)
            .spawn()
            .expect("SocketServer thread spawn() error.");

    // Wait a bit for the server to accept connections.
    std::thread::sleep(Duration::from_millis(20));

    // Make sure the value of `has_been_connected()` is correct.
    assert!(!server.wait_connection(Duration::ZERO));

    // Connect a client successfully to the server.
    let client = SocketClient::new(address)
        .expect("Connection to SocketServer failed.");

    // Wait a bit for the server to send the connection message
    assert!(server.wait_connection(Duration::from_millis(20)));
    (client, server)
}
