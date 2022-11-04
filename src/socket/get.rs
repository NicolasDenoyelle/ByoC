use super::message::{Request, Response};
use super::SocketClient;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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
        match self.process_request(Request::<K, V>::Get(key.clone())) {
            Response::Get(Some(v)) => Some(LifeTimeGuard::new(Rc::new(v))),
            Response::Get(None) => None,
            _ => mismatch_panic(),
        }
    }
}

pub struct SocketClientCell<K, V>
where
    K: Serialize + Clone,
    V: Serialize + Clone,
{
    key: K,
    value: V,
    is_updated: bool,
    stream: Arc<Mutex<TcpStream>>,
    unused: PhantomData<K>,
}

impl<K, V> SocketClientCell<K, V>
where
    K: Serialize + Clone,
    V: Serialize + Clone,
{
    pub fn new(key: K, value: V, stream: Arc<Mutex<TcpStream>>) -> Self {
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
    K: Serialize + Clone,
    V: Serialize + Clone,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K, V> DerefMut for SocketClientCell<K, V>
where
    K: Serialize + Clone,
    V: Serialize + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_updated = true;
        &mut self.value
    }
}

impl<K, V> Drop for SocketClientCell<K, V>
where
    K: Serialize + Clone,
    V: Serialize + Clone,
{
    fn drop(&mut self) {
        if !self.is_updated {
            return;
        }

        let mut stream = self.stream.lock().unwrap();
        let request = Request::<K, V>::WriteBack((
            self.key.clone(),
            self.value.clone(),
        ));
        bincode::serialize_into(&mut *stream, &request).expect(
            "SocketClientCell failed to WriteBack to SocketServer",
        );
    }
}

impl<K, V> GetMut<K, V> for SocketClient
where
    K: DeserializeOwned + Serialize + Clone,
    V: DeserializeOwned + Serialize + Clone,
{
    type Target = SocketClientCell<K, V>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        match self.process_request(Request::<K, V>::GetMut(key.clone())) {
            Response::GetMut(Some(v)) => {
                Some(LifeTimeGuard::new(SocketClientCell::new(
                    key.clone(),
                    v,
                    Arc::clone(&self.stream),
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
