use modular_bitfield::specifiers::*;
use modular_bitfield::*;
use std::alloc::{alloc, Layout};
use std::ptr;

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

impl RleVolume {
    /// Construct volume with no voxels with given size. `ysize` is up direction
    pub fn empty(xsize: u32, ysize: u32, zsize: u32) -> Self {
        // Restrict them to be power of two to allow easy mipmapping
        assert!(is_power2(xsize), "xsize is not power of two!");
        assert!(is_power2(ysize), "ysize is not power of two!");
        assert!(is_power2(zsize), "zsize is not power of two!");
        assert!(xsize < 1024, "xsize of RleVolume is bigger than or equal to 1024!");
        assert!(ysize < 1024, "ysize of RleVolume is bigger than or equal to 1024!");
        assert!(zsize < 1024, "zsize of RleVolume is bigger than or equal to 1024!");

        let pointers;
        unsafe {
            let num_pointers = (xsize*zsize) as usize;
            pointers = alloc(Layout::array::<PointerColumn>(num_pointers).unwrap()) as *mut PointerColumn;
            for i in 0 .. num_pointers {
                let point = pointers.offset(i as isize);
                *point = PointerColumn {
                    pointer: 0,
                    rle_count: 0, 
                    first_range: RleRange::new().with_skipped(ysize as u16).with_drawn(0),
                };
            }
        }

        RleVolume {
            width: xsize,
            height: zsize,
            pointers, 
            columns_size: 0, 
            columns: ptr::null::<u8>() as *mut u8, 
        }
    }
}

/// Trick to check whether the number is power of two. Zero is not counted as true.
fn is_power2(x: u32) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_volumes() {
        let volume1 = RleVolume::empty(1, 1, 1);
        let volume2 = RleVolume::empty(2, 2, 2);
        let volume4 = RleVolume::empty(4, 4, 4);
        let volume8 = RleVolume::empty(8, 8, 8);
        let volume16 = RleVolume::empty(16, 16, 16);
        let volume32 = RleVolume::empty(32, 32, 32);
        let volume64 = RleVolume::empty(64, 64, 64);
        let volume128 = RleVolume::empty(128, 128, 128);
        let volume256 = RleVolume::empty(256, 256, 256);
        let volume512 = RleVolume::empty(512, 512, 512);
    }

    #[test]
    #[should_panic]
    fn empty_invalid_volume0() {
        let v = RleVolume::empty(0, 0, 0);
    }    
    
    #[test]
    #[should_panic]
    fn empty_invalid_volume1() {
        let v = RleVolume::empty(1, 0, 0);
    }

    #[test]
    #[should_panic]
    fn empty_invalid_volume2() {
        let v = RleVolume::empty(0, 1, 0);
    }

    #[test]
    #[should_panic]
    fn empty_invalid_volume3() {
        let v = RleVolume::empty(0, 0, 1);
    }

    #[test]
    #[should_panic]
    fn empty_invalid_volume4() {
        let v = RleVolume::empty(1024, 1024, 1024);
    }

    #[test]
    #[should_panic]
    fn empty_invalid_volume5() {
        let v = RleVolume::empty(1024, 512, 1024);
    }
}
