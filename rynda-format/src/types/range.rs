use modular_bitfield::{specifiers::*, *};
use std::fmt;

/// Amount of bytes `RleRange` takes in packed form
pub const RLE_RANGE_SIZE: usize = 2;
/// Amount of values skipped field can represent
pub const RLE_SKIPPED_MAX: usize = 1024;
/// Amount of values drawn field can represent
pub const RLE_DRAWN_MAX: usize = 64;

/// Single run length encoded range of voxels. First, skip "empty" voxels and then draw N voxels from the buffer.
#[repr(packed(2))]
#[derive(Clone, Copy)]
#[bitfield]
pub struct RleRange {
    /// How many voxels are not drawn in the Y column
    pub skipped: B10,
    /// How many voxels are drawn after skipped part
    pub drawn: B6,
}

impl PartialEq for RleRange {
    fn eq(&self, other: &Self) -> bool {
        self.skipped() == other.skipped() && self.drawn() == other.drawn()
    }
}

impl Eq for RleRange {}

impl fmt::Debug for RleRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RleRange")
            .field("skipped", &self.skipped())
            .field("drawn", &self.drawn())
            .finish()
    }
}

impl RleRange {
    /// Fast construction from skipped and drawn range
    pub fn range(skipped: u16, drawn: u8) -> Self {
        RleRange::new().with_skipped(skipped).with_drawn(drawn)
    }
}
