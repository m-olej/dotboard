
#[macro_export]
macro_rules! define_ipc_messages {
    (
        // Frame tag of module
        frame_tag = $frame:expr;
        
        // Message definitions: name, message_tag, payload (key: type) ! no default values
        $(
            $msg_name:ident ($msg_tag:expr) {
                $(pub $field:ident: $fty:ty),* $(,)?
            }
        )*
    ) => {
        // Each message is resolvable to u8 value
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::TryFromPrimitive)]
        pub enum MessageTag {
            $( $msg_name = $msg_tag, )*
        }

        // Guarantee (de)serializability of message payloads
        $(
            #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
            pub struct $msg_name {
                $(pub $field: $fty,)*
            }

            // Automatically assign appropriate tags
            impl $crate::ipc::translator::IpcMessage for $msg_name {
                const FRAME_TAG: u8 = $frame as u8;
                const MESSAGE_TAG: u8 = MessageTag::$msg_name as u8;
            }
        )*
    };
}
