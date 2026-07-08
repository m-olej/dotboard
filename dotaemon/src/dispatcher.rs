use bytes::{Buf, Bytes};
use dotcore::ipc::registry::Router;
use tokio::sync::{broadcast};
use dotcore::ipc::connection::IpcConnection;
use futures::{SinkExt, StreamExt};

pub struct Dispatcher;

impl Dispatcher {

    // Creates 2 async tasks for concurrent reading and writing
    // Ensures cancellation safety (unlike tokio::select!)
    pub async fn run(
        connection: IpcConnection,
        router: Router,
        mut egress_rx: broadcast::Receiver<Bytes>
    ) {
        
        // tx_sink:     Daemon -> Client
        // rx_stream:   Client -> Daemon
        let (mut tx_sink, mut rx_stream) = connection.into_split();

        // Receiver (Client -> Daemon) //
        // Receives:    tagged serialized frame from TUI
        // Sends:       Message frame including message tag and serialized payload
        tokio::spawn(async move {
            while let Some(result) = rx_stream.next().await {

                let mut raw_payload = match result {
                    Ok(payload) => payload,
                    Err(e) => {eprintln!("Connection closed abruptly: {e}"); break;}
                };

                let module_tag = raw_payload.get_u8();
                let message_tag = raw_payload.get_u8();
                let payload = raw_payload.freeze();

                router.route(module_tag, message_tag, payload).await;
            }
        });

        // Sender (Module -> Daemon -> client) //
        // Receives:    tagged frame serialized by module
        // Sends:       the same unchanged data through UDS to TUI 
        let egress_task = tokio::spawn(async move {
            loop {
                let raw_payload = egress_rx.recv().await.unwrap(); 
                if let Err(e) = tx_sink.send(raw_payload).await {
                    eprintln!("Connection closed abruptly: {e}");
                    break;
                }
            }
        });

        egress_task.abort();
        println!("Dispatcher client connection closed");
    }
}


