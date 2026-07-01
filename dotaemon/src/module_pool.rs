use tokio::sync::mpsc;
use bytes::Bytes;


pub struct ModulePool {
    worker_threads: usize,
    rx_module: mpsc::Receiver<(u8, Bytes)>,
    tx_comm: mpsc::Sender<(u8, Bytes)>
}

impl ModulePool {

pub fn new(
    worker_threads: usize,
    rx_module: mpsc::Receiver<(u8, Bytes)>,
    tx_comm: mpsc::Sender<(u8, Bytes)>
) -> Self {
    Self {
        worker_threads,
            rx_module,
            tx_comm,
        }
    }

    pub fn run(self) {
        // Build async runtime of thread pool
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_threads)
            .enable_all()
            .build()
            .expect("Failed to build module thread pool runtime");

        // Core thread pool loop
        rt.block_on(async move {
            println!("Module Pool: running with {0} threads", self.worker_threads);
        });
    }
}
