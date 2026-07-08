use bytes::{Bytes, BufMut, BytesMut};
use rkyv::{Serialize, api::high::HighSerializer, ser::allocator::ArenaHandle, util::AlignedVec, rancor::Error};
use tokio::net::{UnixListener, UnixStream};
use tokio_util::{codec::{Framed, length_delimited::LengthDelimitedCodec}};
use futures::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use std::io;

use crate::ipc::translator::IpcMessage;

pub const SOCKET_PATH: &str = "/tmp/dotboard.sock";
pub type UnixReadStream = SplitStream<Framed<UnixStream, LengthDelimitedCodec>>;
pub type UnixWriteStream = SplitSink<Framed<UnixStream, LengthDelimitedCodec>, Bytes>;

/// Shared connection object defining the project communication interface
pub struct IpcConnection {
    stream: Framed<UnixStream, LengthDelimitedCodec>
}

impl IpcConnection {
    
    /// Ownership of the stream is moved from client to `IpcConnection`
    pub fn from_stream(stream: UnixStream) -> Self {
        let framed = Framed::new(stream, LengthDelimitedCodec::new());
        IpcConnection {
           stream: framed 
        }
    }

    /// Create `IpcConnection` wrapped `UnixStream`
    pub async fn connect() -> io::Result<Self> {
        let stream = Framed::new(UnixStream::connect(SOCKET_PATH).await?, LengthDelimitedCodec::new());
        Ok (Self { stream })
    }

    pub async fn send_frame<T>(&mut self, event: &T) -> io::Result<()> 
    where 
        T: IpcMessage,
        T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
    {
        let payload_bytes = rkyv::to_bytes::<Error>(event)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let mut buffer = BytesMut::with_capacity(payload_bytes.len() + 2);

        buffer.put_u8(T::FRAME_TAG);
        buffer.put_u8(T::MESSAGE_TAG);
        buffer.put_slice(&payload_bytes);

        self.stream.send(buffer.freeze()).await?;

        Ok(())
    }

    pub fn into_split(self) -> (
        UnixWriteStream,
        UnixReadStream, // returns BytesMut
    ) {
        self.stream.split()
    }
}

/// Factory pattern for the daemon that abstracts socket creatio
pub struct IpcListener {
    listener: UnixListener
}

impl IpcListener {

    /// Create `IpcListener` wrapped `UnixListener`
    pub fn bind(path: &str) -> io::Result<Self> {
        
        // Remove stale socket if present, ignore possible OS 13 error
        let _ = std::fs::remove_file(path);

        let listener = UnixListener::bind(path)?;

        Ok ( IpcListener { listener } )
    }

    pub async fn accept(&self) -> io::Result<IpcConnection> {
        let (stream, addr) = self.listener.accept().await?;

        println!("New connection from {addr:?}");

        Ok ( 
            IpcConnection {
                stream: Framed::new(stream, LengthDelimitedCodec::new())
            }
        )
    }
}
