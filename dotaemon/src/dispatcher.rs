use std::path::PathBuf;
use bytes::Bytes;
use std::thread;
use tokio::sync::mpsc;

pub struct Dispatcher {
    socket_path: PathBuf,
    rx_comm: mpsc::Receiver<(u8, Bytes)>,
    tx_module: mpsc::Sender<(u8, Bytes)>
} 

impl Dispatcher {

    pub fn new(
        socket_path: PathBuf,
        rx_comm: mpsc::Receiver<(u8, Bytes)>,
        tx_module: mpsc::Sender<(u8, Bytes)>
    ) -> Self {
        Self {
            socket_path,
            rx_comm,
            tx_module
        }
    }

    pub fn spawn() {
        // Dedicated OS thread to prevent any work-stealing interruptions
        thread::spawn(move || {
            // Async runtime for I/O interactions with UDS socket
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build dispatcher runtime");

            // Core dispatcher loop
            rt.block_on(async move {
                
            });
        });
    }
}
