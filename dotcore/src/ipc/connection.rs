use bytes::Bytes;
use rkyv::{Serialize, api::high::HighSerializer, ser::allocator::ArenaHandle, util::AlignedVec, rancor::Error};
use tokio::net::{UnixListener, UnixStream};
use tokio_util::{codec::{Framed, length_delimited::LengthDelimitedCodec}};
use futures::{SinkExt, StreamExt};
use std::io;

pub const SOCKET_PATH: &str = "/tmp/dotboard.sock";

/// Shared connection object defining the project communication interface
struct IpcConnection {
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
    pub async fn connect(self, path: &str) -> io::Result<Self> {
        let stream = Framed::new(UnixStream::connect(path).await?, LengthDelimitedCodec::new());
        Ok (Self { stream })
    }

    pub async fn send_event<T>(&mut self, event: &T) -> io::Result<()> 
    where 
        T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
    {
        let aligned_bytes = rkyv::to_bytes::<Error>(event)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        
        let payload = Bytes::from(aligned_bytes.into_vec());

        self.stream.send(payload).await?;

        Ok(())
    }
}

/// Factory pattern for the daemon that abstracts socket creatio
struct IpcListener {
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

    pub async fn accept(self) -> io::Result<IpcConnection> {
        let (stream, addr) = self.listener.accept().await?;

        println!("New connection from {addr:?}");

        Ok ( 
            IpcConnection {
                stream: Framed::new(stream, LengthDelimitedCodec::new())
            }
        )
    }
}
