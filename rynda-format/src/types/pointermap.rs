use super::range::RleRange;

/// Describes head of Y column of voxels in pointers map.
#[repr(packed(8))]
#[derive(Debug)]
pub struct PointerColumn {
    /// Offset in columns array in bytes
    pub pointer: u32,
    /// Count or RLE intervals in the column
    pub rle_count: u16,
    /// First skip-draw range is kept here for optmiziation, as 64 bit is the largest amount
    /// of memory that can be pulled in one read. This strategy increases the rendering performance (speed)
    /// particularly for large outdoor environments and landscape-like scenes with hills and mountains.
    pub first_range: RleRange,
}
