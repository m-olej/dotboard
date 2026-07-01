use bytes::Bytes;
use rkyv::{
    api::high::{from_bytes, HighDeserializer, HighValidator},
    bytecheck::CheckBytes,
    rancor::Error,
    Archive, Deserialize,
};

/// Generic deserialization trait to interact with zero-copy data sent over the UDS
pub trait IpcHandler<P>
where
    P: Archive + Send + 'static,
    // The strict rkyv 0.8 bounds required for safe from_bytes deserialization
    P::Archived: for<'a> CheckBytes<HighValidator<'a, Error>> 
        + Deserialize<P, HighDeserializer<Error>>,
{
    /// The business logic for this specific payload.
    fn handle(payload: P) -> impl std::future::Future<Output = ()> + std::marker::Send;

    /// Spawns the task, aligns the network bytes, and safely deserializes to an owned struct.
    fn spawn_task(payload: Bytes) {
        tokio::spawn(async move {

            // Validation and Deserialization
            match from_bytes::<P, Error>(&payload) {
                Ok(owned_struct) => {
                    // run entrypoint for module reaction to event passing its deserialized struct
                    Self::handle(owned_struct).await;
                }
                Err(e) => {
                    eprintln!("Framework dropped corrupt payload: {e}");
                }
            }
        });
    }
}
