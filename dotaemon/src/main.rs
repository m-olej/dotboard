use bytes::Bytes;
use tokio::sync::mpsc;
use dispatcher::Dispatcher;
use module_pool::ModulePool;
use std::{path::PathBuf, str::FromStr};
use dotcore::ipc::connection::SOCKET_PATH;

mod dispatcher;
mod module_pool;
mod modules;


fn main() {
    println!("Dotaemon starting");

    let (tx_comm, rx_comm) = mpsc::channel::<(u8, Bytes)>(1024);
    let (tx_module, rx_module) = mpsc::channel::<(u8, Bytes)>(1024);

    let socket_path = PathBuf::from_str(SOCKET_PATH).unwrap();

    Dispatcher::new(socket_path, rx_comm, tx_module);
    let dispatcher_handle = Dispatcher::spawn();

    let worker_pool = ModulePool::new(4, rx_module, tx_comm);

    worker_pool.run(); 

    match dispatcher_handle.join() {
        Ok(_) => println!("Dotaemon stopped"),
        Err(e) => eprintln!("Error during shutdown of Dotaemon: {e:?}")
    }
}
