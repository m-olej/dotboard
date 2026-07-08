use crate::define_ipc_messages;
use crate::events::frames::FrameTag;

define_ipc_messages! {
    frame_tag = FrameTag::Tennis;

    Ping(0x01) {
        pub msg: String,
    }

    Pong(0x02) {
        pub reply: String,
    }
}
