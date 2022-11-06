use super::client::process_request;
use super::message::{Request, Response};
use super::SocketClient;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

fn mismatch_panic() -> ! {
    panic!("SocketClient Request and SocketServer Response mismatch.");
}

impl<K, V> Get<K, V> for SocketClient
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize,
{
    type Target = Rc<V>;

    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let response = process_request(
            &mut self.stream,
            Request::<K, V>::Get(key.clone()),
        );
        match response {
            Response::Get(Some(v)) => Some(LifeTimeGuard::new(Rc::new(v))),
            Response::Get(None) => None,
            _ => mismatch_panic(),
        }
    }
}

pub struct SocketClientCell<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    key: K,
    value: V,
    is_updated: bool,
    stream: TcpStream,
    unused: PhantomData<K>,
}

impl<K, V> SocketClientCell<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    pub fn new(key: K, value: V, stream: TcpStream) -> Self {
        SocketClientCell {
            key,
            value,
            is_updated: false,
            stream,
            unused: PhantomData,
        }
    }
}

impl<K, V> Deref for SocketClientCell<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K, V> DerefMut for SocketClientCell<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_updated = true;
        &mut self.value
    }
}

impl<K, V> Drop for SocketClientCell<K, V>
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    fn drop(&mut self) {
        if !self.is_updated {
            return;
        }
        let request = Request::<K, V>::WriteBack((
            self.key.clone(),
            self.value.clone(),
        ));
        let response = process_request(&mut self.stream, request);
        if let Response::Error(e) = response {
            panic!(
                "Error writing back SocketClient GetMut value: {:?}",
                e
            );
        }
    }
}

impl<K, V> GetMut<K, V> for SocketClient
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    type Target = SocketClientCell<K, V>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let response = process_request(
            &mut self.stream,
            Request::<K, V>::GetMut(key.clone()),
        );
        match response {
            Response::GetMut(Some(v)) => {
                Some(LifeTimeGuard::new(SocketClientCell::new(
                    key.clone(),
                    v,
                    self.stream
                        .try_clone()
                        .expect("Error cloning TcpStream"),
                )))
            }
            Response::GetMut(None) => None,
            _ => mismatch_panic(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::tests::make_server_client;
    use crate::tests::{test_get, test_get_mut};

    #[test]
    fn test_server_client_get() {
        let (client, server) = make_server_client(10, "localhost:6374");
        test_get(client);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_empty_server_client_get() {
        let (client, server) = make_server_client(0, "localhost:6373");
        test_get(client);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_server_client_get_mut() {
        let (client, server) = make_server_client(10, "localhost:6372");
        test_get_mut(client);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_empty_server_client_get_mut() {
        let (client, server) = make_server_client(0, "localhost:6371");
        test_get_mut(client);
        server.stop_and_join().unwrap();
    }
}
