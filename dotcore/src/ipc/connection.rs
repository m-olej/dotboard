use tokio::net::{UnixListener, UnixStream};
use tokio_util::{codec::{Framed, length_delimited::LengthDelimitedCodec}};
use std::io;

const SOCKET_PATH: &str = "/tmp/dotboard.sock";

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
    pub async fn connect(self) -> io::Result<Self> {
        let stream = Framed::new(UnixStream::connect(SOCKET_PATH).await?, LengthDelimitedCodec::new());
        Ok (Self { stream })
    }
}

/// Factory pattern for the daemon that abstracts socket creatio
struct IpcListener {
    listener: UnixListener
}

impl IpcListener {

    /// Create `IpcListener` wrapped `UnixListener`
    pub fn bind() -> io::Result<Self> {
        
        // Remove stale socket if present, ignore possible OS 13 error
        let _ = std::fs::remove_file(SOCKET_PATH);

        let listener = UnixListener::bind(SOCKET_PATH)?;

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
