use bytes::Bytes;
use rkyv::{
    api::high::{from_bytes, HighDeserializer, HighValidator},
    bytecheck::CheckBytes,
    rancor::Error,
    Archive, Deserialize,
};
use tokio::sync::broadcast;

/// Generic deserialization trait to interact with zero-copy data sent over the UDS
pub trait IpcHandler<P>
where
    P: Archive + Send + 'static,
    P::Archived: for<'a> CheckBytes<HighValidator<'a, Error>> 
        + Deserialize<P, HighDeserializer<Error>>,
{
    /// The business logic for this specific payload.
    fn handle(payload: P, egress_tx: broadcast::Sender<Bytes>) -> impl std::future::Future<Output = ()> + std::marker::Send;

    /// Spawns the task, aligns the network bytes, and safely deserializes to an owned struct.
    fn spawn_task(payload: Bytes, egress_tx: broadcast::Sender<Bytes>) {
        tokio::spawn(async move {

            // Validation and Deserialization
            match from_bytes::<P, Error>(&payload) {
                Ok(owned_struct) => {
                    // run entrypoint for module reaction to event passing its deserialized struct
                    Self::handle(owned_struct, egress_tx).await;
                }
                Err(e) => {
                    eprintln!("Framework dropped corrupt payload: {e}");
                }
            }
        });
    }
}


pub trait IpcMessage {
    const FRAME_TAG: u8;
    const MESSAGE_TAG: u8;
}
