use super::error::ResponseError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cmp::PartialEq;

pub trait Message: Sized {
    fn send<W: std::io::Write + ?Sized>(
        &self,
        stream: &mut W,
    ) -> bincode::Result<()>;

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self>;
}

trait MessageHeader: Sized {}

impl<T: MessageHeader + std::fmt::Debug> Message for T {
    fn send<W: std::io::Write + ?Sized>(
        &self,
        stream: &mut W,
    ) -> bincode::Result<()> {
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        };
        println!("Sending Header: {:?}:{:?}", self, slice);

        stream
            .write_all(slice)
            .map_err(|e| Box::new(bincode::ErrorKind::Io(e)))
    }

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self> {
        let mut uninit = std::mem::MaybeUninit::<Self>::uninit();
        let buf: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                uninit.as_mut_ptr() as *mut u8,
                std::mem::size_of::<Self>(),
            )
        };

        if let Err(ioerror) = stream.read_exact(buf) {
            return Err(Box::new(bincode::ErrorKind::Io(ioerror)));
        }

        println!("Received: {:?}", buf);
        let header = unsafe { uninit.assume_init() };
        println!("Received Header: {:?}", &header);
        Ok(header)
    }
}

fn deserialize_into<R: std::io::Read + ?Sized, V: DeserializeOwned>(
    stream: &mut R,
    size: usize,
) -> bincode::Result<V> {
    let mut buf = vec![0u8; size];
    stream
        .read_exact(buf.as_mut_slice())
        .map_err(|ioerror| Box::new(bincode::ErrorKind::Io(ioerror)))?;
    println!("Received {}: {:?}", std::any::type_name::<V>(), buf);
    bincode::deserialize(buf.as_slice())
}

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug)]
pub(super) enum RequestHeader {
    Capacity,
    Size,
    Contains(usize),
    Take(usize),
    TakeMultiple(usize),
    Pop(usize),
    Push(usize),
    Flush,
    Get(usize),
    GetMut(usize),
    WriteBack(usize),
}

impl MessageHeader for RequestHeader {}

impl RequestHeader {
    pub fn from_request<K: Serialize, V: Serialize>(
        request: &Request<K, V>,
    ) -> bincode::Result<Self> {
        match request {
            Request::Capacity => Ok(Self::Capacity),
            Request::Size => Ok(Self::Size),
            Request::Pop(size) => Ok(Self::Pop(*size)),
            Request::Flush => Ok(Self::Flush),
            Request::Contains(k) => {
                Ok(Self::Contains(bincode::serialized_size(k)? as usize))
            }
            Request::Take(k) => {
                Ok(Self::Take(bincode::serialized_size(k)? as usize))
            }
            Request::TakeMultiple(vec) => Ok(Self::TakeMultiple(
                bincode::serialized_size(vec)? as usize,
            )),
            Request::Push(vec) => {
                Ok(Self::Push(bincode::serialized_size(vec)? as usize))
            }
            Request::Get(k) => {
                Ok(Self::Get(bincode::serialized_size(k)? as usize))
            }
            Request::GetMut(k) => {
                Ok(Self::GetMut(bincode::serialized_size(k)? as usize))
            }
            Request::WriteBack(kv) => {
                Ok(Self::WriteBack(bincode::serialized_size(kv)? as usize))
            }
        }
    }
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
        let header = RequestHeader::from_request(self)?;
        header.send(stream)?;

        match self {
            Self::Capacity | Self::Size | Self::Pop(_) | Self::Flush => {
                Ok(())
            }
            Self::Contains(val) => bincode::serialize_into(stream, val),
            Self::Take(val) => bincode::serialize_into(stream, val),
            Self::TakeMultiple(val) => {
                bincode::serialize_into(stream, val)
            }
            Self::Push(val) => bincode::serialize_into(stream, val),
            Self::Get(val) => bincode::serialize_into(stream, val),
            Self::GetMut(val) => bincode::serialize_into(stream, val),
            Self::WriteBack(val) => bincode::serialize_into(stream, val),
        }
    }

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self> {
        let header = RequestHeader::receive(stream)?;
        match header {
            RequestHeader::Capacity => Ok(Self::Capacity),
            RequestHeader::Size => Ok(Self::Size),
            RequestHeader::Flush => Ok(Self::Flush),
            RequestHeader::Pop(s) => Ok(Self::Pop(s)),
            RequestHeader::Contains(s) => {
                deserialize_into(stream, s).map(|v| Self::Contains(v))
            }
            RequestHeader::Take(s) => {
                deserialize_into(stream, s).map(|v| Self::Take(v))
            }
            RequestHeader::TakeMultiple(s) => {
                deserialize_into(stream, s).map(|v| Self::TakeMultiple(v))
            }
            RequestHeader::Push(s) => {
                deserialize_into(stream, s).map(|v| Self::Push(v))
            }
            RequestHeader::Get(s) => {
                deserialize_into(stream, s).map(|v| Self::Get(v))
            }
            RequestHeader::GetMut(s) => {
                deserialize_into(stream, s).map(|v| Self::GetMut(v))
            }
            RequestHeader::WriteBack(s) => {
                deserialize_into(stream, s).map(|v| Self::WriteBack(v))
            }
        }
    }
}

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug)]
pub(super) enum ResponseHeader {
    Capacity(usize),
    Size(usize),
    Contains(bool),
    WriteBackAcknowledgment,
    Error(ResponseError),
    Take(usize),
    TakeMultiple(usize),
    Pop(usize),
    Push(usize),
    Flush(usize),
    Get(usize),
    GetMut(usize),
}

impl MessageHeader for ResponseHeader {}

impl ResponseHeader {
    pub fn from_response<K: Serialize, V: Serialize>(
        response: &Response<K, V>,
    ) -> bincode::Result<Self> {
        match response {
            Response::Capacity(s) => Ok(ResponseHeader::Capacity(*s)),
            Response::Size(s) => Ok(ResponseHeader::Size(*s)),
            Response::Contains(s) => Ok(ResponseHeader::Contains(*s)),
            Response::Error(s) => Ok(ResponseHeader::Error(*s)),
            Response::WriteBackAcknowledgment => {
                Ok(ResponseHeader::WriteBackAcknowledgment)
            }
            Response::Take(val) => {
                Ok(Self::Take(bincode::serialized_size(val)? as usize))
            }
            Response::TakeMultiple(val) => Ok(Self::TakeMultiple(
                bincode::serialized_size(val)? as usize,
            )),
            Response::Pop(val) => {
                Ok(Self::Pop(bincode::serialized_size(val)? as usize))
            }
            Response::Push(val) => {
                Ok(Self::Push(bincode::serialized_size(val)? as usize))
            }
            Response::Flush(val) => {
                Ok(Self::Flush(bincode::serialized_size(val)? as usize))
            }
            Response::Get(val) => {
                Ok(Self::Get(bincode::serialized_size(val)? as usize))
            }
            Response::GetMut(val) => {
                Ok(Self::GetMut(bincode::serialized_size(val)? as usize))
            }
        }
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
        let header = ResponseHeader::from_response(self)?;
        header.send(stream)?;

        match self {
            Self::Capacity(_)
            | Self::Size(_)
            | Self::Contains(_)
            | Self::WriteBackAcknowledgment
            | Self::Error(_) => Ok(()),
            Self::Take(val) => bincode::serialize_into(stream, val),
            Self::TakeMultiple(val) => {
                bincode::serialize_into(stream, val)
            }
            Self::Pop(val) => bincode::serialize_into(stream, val),
            Self::Push(val) => bincode::serialize_into(stream, val),
            Self::Flush(val) => bincode::serialize_into(stream, val),
            Self::Get(val) => bincode::serialize_into(stream, val),
            Self::GetMut(val) => bincode::serialize_into(stream, val),
        }
    }

    fn receive<R: std::io::Read + ?Sized>(
        stream: &mut R,
    ) -> bincode::Result<Self> {
        let header = ResponseHeader::receive(stream)?;

        match header {
            ResponseHeader::Capacity(s) => Ok(Self::Capacity(s)),
            ResponseHeader::Size(s) => Ok(Self::Size(s)),
            ResponseHeader::Contains(b) => Ok(Self::Contains(b)),
            ResponseHeader::WriteBackAcknowledgment => {
                Ok(Self::WriteBackAcknowledgment)
            }
            ResponseHeader::Error(err) => Ok(Self::Error(err)),
            ResponseHeader::Take(s) => {
                deserialize_into(stream, s).map(|v| Self::Take(v))
            }
            ResponseHeader::TakeMultiple(s) => {
                deserialize_into(stream, s).map(|v| Self::TakeMultiple(v))
            }
            ResponseHeader::Pop(s) => {
                deserialize_into(stream, s).map(|v| Self::Pop(v))
            }
            ResponseHeader::Push(s) => {
                deserialize_into(stream, s).map(|v| Self::Push(v))
            }
            ResponseHeader::Flush(s) => {
                deserialize_into(stream, s).map(|v| Self::Flush(v))
            }
            ResponseHeader::Get(s) => {
                deserialize_into(stream, s).map(|v| Self::Get(v))
            }
            ResponseHeader::GetMut(s) => {
                deserialize_into(stream, s).map(|v| Self::GetMut(v))
            }
        }
    }
}

#[cfg(all(test, feature = "stream"))]
mod tests {
    use super::{
        Message, Request, RequestHeader, Response, ResponseHeader,
    };
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
    fn test_request_header() {
        test_message(RequestHeader::Capacity);
        test_message(RequestHeader::Size);
        test_message(RequestHeader::Contains(0usize));
        test_message(RequestHeader::Take(0usize));
        test_message(RequestHeader::TakeMultiple(0usize));
        test_message(RequestHeader::Pop(0usize));
        test_message(RequestHeader::Push(0usize));
        test_message(RequestHeader::Flush);
        test_message(RequestHeader::Get(0usize));
        test_message(RequestHeader::GetMut(0usize));
        test_message(RequestHeader::WriteBack(0usize));
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
    fn test_response_header() {
        test_message(ResponseHeader::Capacity(0usize));
        test_message(ResponseHeader::Size(0usize));
        test_message(ResponseHeader::Contains(false));
        test_message(ResponseHeader::WriteBackAcknowledgment);
        test_message(ResponseHeader::Error(
            ResponseError::InvalidWriteBackKey,
        ));
        test_message(ResponseHeader::Take(0usize));
        test_message(ResponseHeader::TakeMultiple(0usize));
        test_message(ResponseHeader::Get(0usize));
        test_message(ResponseHeader::GetMut(0usize));
        test_message(ResponseHeader::Pop(0usize));
        test_message(ResponseHeader::Push(0usize));
        test_message(ResponseHeader::Flush(0usize));
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
            ResponseError::InvalidWriteBackKey,
        ));
    }
}
