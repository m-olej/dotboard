use tokio::sync::{broadcast, mpsc};
use bytes::Bytes;

pub trait DotModule {
    // Initialize module runtime (message handler)
    fn init(module_rx: mpsc::Receiver<(u8, Bytes)>, egress_tx: broadcast::Sender<Bytes>) -> impl Future<Output = ()> + Send;
}
