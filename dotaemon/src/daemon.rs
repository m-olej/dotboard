use bytes::Bytes;
use dotcore::ipc::{connection::{IpcListener, SOCKET_PATH}, registry::Router};
use tokio::{runtime, sync::broadcast};

use crate::dispatcher::Dispatcher;


pub struct Daemon;

impl Daemon {
    
    pub fn run(
        egress_tx: broadcast::Sender<Bytes>,
        router: Router
    ) {
        // Create dedicated daemon runtime
        let daemon_rt = runtime::Builder::new_current_thread()
            .name("daemon-rt")
            .enable_all()
            .build()
            .expect("Failed to create daemon runtime");

        
        daemon_rt.block_on(async move {
            
            let listener = IpcListener::bind(SOCKET_PATH).expect("Failed to create IpcListener");
            println!("Daemon listening on {SOCKET_PATH}");
            
            loop {
               match listener.accept().await {
                    Ok(connection) => {
                        // increases atomic reference count of shared objects between multiple TUI
                        // instances
                        let router_clone = router.clone(); 
                        let client_egress_rx = egress_tx.subscribe();

                        tokio::spawn(async move {
                            Dispatcher::run(connection, router_clone, client_egress_rx).await;
                        });
                    },
                    Err(e) => eprintln!("Failed to establish UDS connection: {e}")
                } 
            }
        });
    }
}
