use super::client::process_request;
use super::message::{Request, Response};
use super::SocketClient;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

fn mismatch_panic() -> ! {
    panic!("SocketClient Request and SocketServer Response mismatch.");
}

impl<'a, K, V> BuildingBlock<'a, K, V> for SocketClient
where
    K: 'a + DeserializeOwned + Serialize + Clone,
    V: 'a + DeserializeOwned + Serialize,
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

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        match process_request(&self.stream, Request::<K, V>::Flush) {
            Response::Flush(ret) => Box::new(ret.into_iter()),
            _ => mismatch_panic(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::tests::make_server_client;
    use crate::tests::test_building_block;

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
