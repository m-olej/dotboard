use dotcore::{events::frames::FrameTag, ipc::{connection::{IpcListener, SOCKET_PATH}, registry::ModuleRegistry}, modules::interface::DotModule};
use tokio::sync::{mpsc, broadcast};
use bytes::Bytes;


// import modules //
use modules::tennis::TennisModule;
use crate::{daemon::Daemon, module_pool::ModulePool};

mod daemon;
mod dispatcher;
mod module_pool;
mod modules;


fn main() {
    println!("Dotaemon starting");

    // Global egrees bus
    let (global_egress_tx, _) = broadcast::channel::<Bytes>(1024);

    let mut registry = ModuleRegistry::new(global_egress_tx.clone());

    // Register modules for communication //

    let (tennis_rx, tennis_tx) = registry.register(FrameTag::Tennis as u8, 128);
    
    // Register modules for communication //

    let router = registry.build();

    // Runs in the background
    let mut module_pool = ModulePool::new(4);
    module_pool.build();

    // Place modules into the ModulePool runtime

    module_pool.add(TennisModule::init(tennis_rx, tennis_tx));

    // Place modules into the ModulePool runtime

    Daemon::run(global_egress_tx, router);

}
