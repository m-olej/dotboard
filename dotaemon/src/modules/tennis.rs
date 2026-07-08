use dotcore::{events::{frames::FrameTag, tennis::{Ping, Pong, MessageTag}}, ipc::translator::IpcHandler, modules::interface::DotModule};
use tokio::sync::{broadcast, mpsc};
use bytes::Bytes;

// Define modules private fields
pub struct TennisModule {
    egress_tx: broadcast::Sender<Bytes> 
}

// Define modules initialization
impl DotModule for TennisModule {

    // Practically a constructor
    async fn init(mut module_rx: mpsc::Receiver<(u8, Bytes)>, egress_rx: broadcast::Sender<Bytes>) {
               
        loop {
            if let Some((tag, rkyv_payload)) = module_rx.recv().await {
                // Decoding happens here  

                match MessageTag::try_from(tag) {
                    Ok(MessageTag::Ping) => <Self as IpcHandler<Ping>>::spawn_task(rkyv_payload, egress_rx.clone()),
                    Ok(MessageTag::Pong) => <Self as IpcHandler<Pong>>::spawn_task(rkyv_payload, egress_rx.clone()),
                    Err(_) => eprintln!("Module doesn't recognize message tag: 0x{tag}")
                }
            
            } else {
               eprintln!("Module failed to receive event");
            }

        }

    }
}

// Implement reactions for Module specific payloads
impl IpcHandler<Ping> for TennisModule {
    async fn handle(payload: Ping, egress_tx: broadcast::Sender<Bytes>) {
        println!("Received a Ping message: {}", payload.msg);
        
    }
}

impl IpcHandler<Pong> for TennisModule {
    async fn handle(payload: Pong, egress_tx: broadcast::Sender<Bytes>) {
        
        println!("Received a Pong message: {}", payload.reply);
    }
}
