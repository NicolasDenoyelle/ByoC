use super::client::process_request;
use super::message::{Request, Response};
use super::SocketClient;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

fn mismatch_panic() -> ! {
    panic!("SocketClient Request and SocketServer Response mismatch.");
}

impl<K, V> BuildingBlock<K, V> for SocketClient<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize,
{
    fn capacity(&self) -> usize {
        match process_request(&self.stream, Request::<K, V>::Capacity) {
            Response::Capacity(s) => s,
            _ => mismatch_panic(),
        }
    }

    fn size(&self) -> usize {
        match process_request(&self.stream, Request::<K, V>::Size) {
            Response::Size(s) => s,
            _ => mismatch_panic(),
        }
    }

    fn contains(&self, key: &K) -> bool {
        match process_request(
            &self.stream,
            Request::<K, V>::Contains(key.clone()),
        ) {
            Response::Contains(tf) => tf,
            _ => mismatch_panic(),
        }
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        match process_request(&self.stream, Request::<K, V>::Pop(n)) {
            Response::Pop(ret) => ret,
            _ => mismatch_panic(),
        }
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        match process_request(
            &self.stream,
            Request::<K, V>::Push(elements),
        ) {
            Response::Push(ret) => ret,
            _ => mismatch_panic(),
        }
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match process_request(
            &self.stream,
            Request::<K, V>::Take(key.clone()),
        ) {
            Response::Take(ret) => ret,
            _ => mismatch_panic(),
        }
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        match process_request(
            &self.stream,
            Request::<K, V>::TakeMultiple(keys.to_vec()),
        ) {
            Response::TakeMultiple(ret) => ret,
            _ => mismatch_panic(),
        }
    }

    type FlushIterator = std::vec::IntoIter<(K, V)>;
    fn flush(&mut self) -> Self::FlushIterator {
        match process_request(&self.stream, Request::<K, V>::Flush) {
            Response::Flush(ret) => ret.into_iter(),
            _ => mismatch_panic(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{test_building_block, TestKey, TestValue};
    use crate::utils::socket::{ServerThreadBuilder, ServerThreadHandle};
    use crate::{Array, Sequential, SocketClient};
    use std::time::Duration;

    fn make_server_client(
        capacity: usize,
        address: &str,
    ) -> (
        SocketClient<TestKey, TestValue>,
        ServerThreadHandle<TestKey, TestValue>,
    ) {
        let container =
            Sequential::new(Array::<(TestKey, TestValue)>::new(capacity));
        let server =
            ServerThreadBuilder::new(String::from(address), container)
                .spawn()
                .expect("SocketServer thread spawn() error.");

        // Wait a bit for the server to accept connections.
        std::thread::sleep(Duration::from_millis(20));

        // Connect a client successfully to the server.
        let client = SocketClient::new(address)
            .expect("Connection to SocketServer failed.");

        // Wait a bit for the server to acknowledge the connection
        std::thread::sleep(Duration::from_millis(20));
        (client, server)
    }

    #[test]
    fn test_server_client_building_block() {
        let (client, server) = make_server_client(10, "localhost:6384");
        test_building_block(client, true);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_empty_server_client_building_block() {
        let (client, server) = make_server_client(0, "localhost:6383");
        test_building_block(client, true);
        server.stop_and_join().unwrap();
    }
}
