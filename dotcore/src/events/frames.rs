use num_enum::TryFromPrimitive;

/// Unique tags that dictate which module will receive the data
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum FrameTag {
    Tennis = 0x00,
    Test = 0x01
}
