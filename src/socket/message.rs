use super::error::ResponseError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cmp::PartialEq;

pub trait Message: Sized {
    fn send<W: std::io::Write>(
        &self,
        stream: &mut W,
    ) -> bincode::Result<()>;

    fn receive<R: std::io::Read>(stream: &mut R) -> bincode::Result<Self>;
}

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug)]
pub enum Request<K, V> {
    Capacity,
    Size,
    Contains(K),
    Take(K),
    TakeMultiple(Vec<K>),
    Pop(usize),
    Push(Vec<(K, V)>),
    Flush,
    Get(K),
    GetMut(K),
    WriteBack((K, V)),
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned>
    Message for Request<K, V>
{
    fn send<W: std::io::Write + ?Sized>(
        &self,
        stream: &mut W,
    ) -> bincode::Result<()> {
        bincode::serialize_into(stream, self)
    }

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self> {
        bincode::deserialize_from(stream)
    }
}

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug)]
pub(super) enum Response<K, V> {
    Capacity(usize),
    Size(usize),
    Contains(bool),
    Take(Option<(K, V)>),
    TakeMultiple(Vec<(K, V)>),
    Pop(Vec<(K, V)>),
    Push(Vec<(K, V)>),
    Flush(Vec<(K, V)>),
    Get(Option<V>),
    GetMut(Option<V>),
    WriteBackAcknowledgment,
    Error(ResponseError),
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned>
    Message for Response<K, V>
{
    fn send<W: std::io::Write + ?Sized>(
        &self,
        stream: &mut W,
    ) -> bincode::Result<()> {
        bincode::serialize_into(stream, self)
    }

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self> {
        bincode::deserialize_from(stream)
    }
}

#[cfg(all(test, feature = "stream"))]
mod tests {
    use super::{Message, Request, Response};
    use crate::socket::error::ResponseError;
    use crate::stream::VecStream;

    fn test_message<T: Message + PartialEq + Eq + std::fmt::Debug>(
        message: T,
    ) {
        let mut write_buf = VecStream::new();
        let mut read_buf = write_buf.clone();

        message
            .send(&mut write_buf)
            .expect("Message serialization error.");

        let recv = T::receive(&mut read_buf)
            .expect("Message deserialization error.");

        assert_eq!(message, recv);
    }

    #[test]
    fn test_request() {
        test_message(Request::<(), ()>::Capacity);
        test_message(Request::<(), ()>::Size);
        test_message(Request::<usize, ()>::Contains(0usize));
        test_message(Request::<usize, ()>::Take(0usize));
        test_message(Request::<(), ()>::TakeMultiple(Vec::new()));
        test_message(Request::<(), ()>::Pop(0usize));
        test_message(Request::<(), ()>::Push(Vec::new()));
        test_message(Request::<(), ()>::Flush);
        test_message(Request::<usize, ()>::Get(0usize));
        test_message(Request::<usize, ()>::GetMut(0usize));
        test_message(Request::<usize, ()>::WriteBack((0usize, ())));
    }

    #[test]
    fn test_response() {
        test_message(Response::<(), ()>::Capacity(0usize));
        test_message(Response::<(), ()>::Size(0usize));
        test_message(Response::<(), ()>::Contains(true));
        test_message(Response::<(), ()>::Take(None));
        test_message(Response::<(), ()>::TakeMultiple(Vec::new()));
        test_message(Response::<(), ()>::Pop(Vec::new()));
        test_message(Response::<(), ()>::Flush(Vec::new()));
        test_message(Response::<(), ()>::Get(None));
        test_message(Response::<(), ()>::GetMut(None));
        test_message(Response::<(), ()>::WriteBackAcknowledgment);
        test_message(Response::<(), ()>::Error(
            ResponseError::WriteBackWithNoOutgoing,
        ));
    }
}
