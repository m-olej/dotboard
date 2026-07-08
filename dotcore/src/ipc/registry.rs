use bytes::Bytes;
use tokio::sync::{broadcast, mpsc};
use std::array;

#[derive(Clone)]
pub struct Router {
    // 256 possible modules, based on tags u8 value
    routes: [Option<mpsc::Sender<(u8, Bytes)>>; 256],
}

impl Router {

    #[inline(always)]
    pub async fn route(&self, module_tag: u8, message_tag: u8, payload: Bytes) {
        if let Some(sender) = &self.routes[module_tag as usize] {
            if sender.send((message_tag, payload)).await.is_err() {
                eprintln!("Failed to route: Module 0x{:02X} is dead", module_tag);
            }
        } else {
            eprintln!("Dropped frame: No module registered for tag 0x{:02X}", module_tag);
        }
    }
}

pub struct ModuleRegistry {
    routes: [Option<mpsc::Sender<(u8, Bytes)>>; 256],
    egress_tx: broadcast::Sender<Bytes>, // The global egress bus to dispatcher
}

impl ModuleRegistry {
    pub fn new(egress_tx: broadcast::Sender<Bytes>) -> Self {
        Self {
            // Initialize empty array
            routes: array::from_fn(|_| None),
            egress_tx,
        }
    }

    /// Registers a module, creates its channel, and returns the pieces it needs to start
    pub fn register(&mut self, tag: u8, buffer_size: usize) -> (mpsc::Receiver<(u8, Bytes)>, broadcast::Sender<Bytes>) {
        if self.routes[tag as usize].is_some() {
            panic!("Module tag 0x{:02X} is already registered!", tag);
        }

        // 1. Create the specific channel for this module
        let (tx_module, rx_module) = mpsc::channel(buffer_size);
        
        // 2. Save the sender in our routing table
        self.routes[tag as usize] = Some(tx_module);

        // 3. Return the receiver and a clone of the global egress bus
        (rx_module, self.egress_tx.clone())
    }

    /// Consumes the builder and returns the read-only Router for the Dispatcher
    pub fn build(self) -> Router {
        Router { routes: self.routes }
    }
}
