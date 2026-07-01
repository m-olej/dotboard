use dotcore::{events::pingpong::PingPong, ipc::translator::IpcHandler};

pub struct PingPongModule {}

impl IpcHandler<PingPong> for PingPongModule {
    
    async fn handle(payload: PingPong) {
        
    }
}
