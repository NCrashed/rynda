use super::{column::RleColumn, pointermap::PointerColumn, range::RleRange, voxel::RgbVoxel};
use ndarray::{s, Array3, Axis};
use std::alloc::{alloc, Layout};
use std::ptr;

/// Run length encoded volume of voxels that consists of two parts. Flat pointers map
/// and columns buffer itself.
#[derive(Debug)]
pub struct RleVolume {
    /// Size of volume by X axis. Number of columns in X axis of pointers buffer.
    pub xsize: u32,
    /// Size of volume by Y axis. Maximum height of XZ columns.
    pub ysize: u32,
    /// Size of volume by Z axis. Number of columns in Z axis of pointers buffer.
    pub zsize: u32,
    /// Contains xsize*zsize elements that defines begining of RLE columns of voxel.
    pub pointers: *mut PointerColumn,
    /// Size of columns buffer in bytes, used for fast copying the volume.
    pub columns_size: u32,
    /// Raw buffer that consists of repeated pattern:
    /// - (rle_count-1)*`RleRange`, where rle_count is taken from first_range in corresponding `PointerColumn`
    /// - N*`RgbVoxel`, where N is calculated of summ of drawn voxels from all `RleRanges` in the column.
    /// It is packed array of `RleColumn` structures.
    pub columns: *mut u8,
}

impl RleVolume {
    /// Construct volume with no voxels with given size. `ysize` is up direction
    pub fn empty(xsize: usize, ysize: usize, zsize: usize) -> Self {
        // Restrict them to be power of two to allow easy mipmapping
        assert!(is_power2(xsize), "xsize is not power of two!");
        assert!(is_power2(ysize), "ysize is not power of two!");
        assert!(is_power2(zsize), "zsize is not power of two!");
        assert!(
            xsize < 1024,
            "xsize of RleVolume is bigger than or equal to 1024!"
        );
        assert!(
            ysize < 1024,
            "ysize of RleVolume is bigger than or equal to 1024!"
        );
        assert!(
            zsize < 1024,
            "zsize of RleVolume is bigger than or equal to 1024!"
        );

        let pointers;
        unsafe {
            let num_pointers = (xsize * zsize) as usize;
            pointers =
                alloc(Layout::array::<PointerColumn>(num_pointers).unwrap()) as *mut PointerColumn;
            for i in 0..num_pointers {
                let point = pointers.offset(i as isize);
                *point = PointerColumn {
                    pointer: 0,
                    rle_count: 0,
                    first_range: RleRange::new().with_skipped(ysize as u16).with_drawn(0),
                };
            }
        }

        RleVolume {
            xsize: xsize as u32,
            ysize: ysize as u32,
            zsize: zsize as u32,
            pointers,
            columns_size: 0,
            columns: ptr::null::<u8>() as *mut u8,
        }
    }
}

/// Trick to check whether the number is power of two. Zero is not counted as true.
fn is_power2(x: usize) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}

impl From<Array3<RgbVoxel>> for RleVolume {
    fn from(array: Array3<RgbVoxel>) -> Self {
        let (xsize, ysize, zsize) = array.dim();
        // Restrict them to be power of two to allow easy mipmapping
        assert!(is_power2(xsize), "xsize is not power of two!");
        assert!(is_power2(ysize), "ysize is not power of two!");
        assert!(is_power2(zsize), "zsize is not power of two!");
        assert!(
            xsize < 1024,
            "xsize of RleVolume is bigger than or equal to 1024!"
        );
        assert!(
            ysize < 1024,
            "ysize of RleVolume is bigger than or equal to 1024!"
        );
        assert!(
            zsize < 1024,
            "zsize of RleVolume is bigger than or equal to 1024!"
        );

        let pointers;
        let mut columns: Vec<RleColumn> = vec![];
        let mut columns_offset: usize = 0;
        unsafe {
            let num_pointers = (xsize * zsize) as usize;
            pointers =
                alloc(Layout::array::<PointerColumn>(num_pointers).unwrap()) as *mut PointerColumn;
            for i in 0..num_pointers {
                let x = i % xsize;
                let z = i / zsize;
                let column = array
                    .slice(s![xsize..xsize + 1, .., zsize..zsize + 1])
                    .remove_axis(Axis(2))
                    .remove_axis(Axis(0));
                let rle_col = RleColumn::compress(column.as_slice().unwrap());
                let (first_range, rest_column) = rle_col.split_head().unwrap();
                let point = pointers.offset(i as isize);
                let rle_count = rest_column.intervals_count();
                assert!(
                    rle_count < 65536,
                    "RLE intervals overflow in single column, expected less than {:?}, got {:?}",
                    65536,
                    rle_count
                );
                *point = PointerColumn {
                    pointer: columns_offset as u32,
                    rle_count: rle_count as u16,
                    first_range,
                };
                columns.push(rest_column.clone());
                columns_offset += rest_column.memory_size();
            }
        }

        let columns_array;
        unsafe {
            columns_array = alloc(Layout::array::<u8>(columns_offset).unwrap());
            let mut offset: usize = 0;
            for c in columns {
                let start = columns_array.offset(offset as isize);
                offset += c.pack_into(start);
            }
        }

        RleVolume {
            xsize: xsize as u32,
            ysize: ysize as u32,
            zsize: zsize as u32,
            pointers,
            columns_size: columns_offset as u32,
            columns: columns_array,
        }
    }
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
