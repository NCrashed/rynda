use modular_bitfield::*;
use modular_bitfield::specifiers::*;

/// Run length encoded volume of voxels that consists of two parts. Flat pointers map 
/// and columns buffer itself. 
pub struct RleVolume {
    /// Width of volume by X axis. Number of columns in X axis of pointers buffer.
    pub width: u32, 
    /// Height of volume by Z axis. Number of columns in Z axis of pointers buffer. 
    pub height: u32,
    /// Contains width*height elements that defines begining of RLE columns of voxel.
    pub pointers: *mut PointerColumn,
    /// Size of columns buffer in bytes, used for fast copying the volume.
    pub columns_size: u32, 
    /// Raw buffer that consists of repeated pattern: 
    /// - (rle_count-1)*`RleRange`, where rle_count is taken from first_range in corresponding `PointerColumn`
    /// - N*`RgbVoxel`, where N is calculated of summ of drawn voxels from all `RleRanges` in the column. 
    pub columns: *mut u8,
}

#[repr(packed(8))]
/// Describes head of Y column of voxels in pointers map.
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

#[repr(packed(2))]
#[bitfield]
/// Single run length encoded range of voxels. First, skip "empty" voxels and then draw N voxels from the buffer.
pub struct RleRange {
    /// How many voxels are not drawn in the Y column
    pub skipped: B10,
    /// How many voxels are drawn after skipped part
    pub drawn: B6,
}

#[repr(packed(2))]
#[bitfield]
/// 16-bit RGB voxel with 5 bits for red and blue colors and 6 bits for green color. Human eye is considered 
/// more sensitive to green tones.
pub struct RgbVoxel {
    pub red: B5, 
    pub green: B6, 
    pub blue: B5,
}