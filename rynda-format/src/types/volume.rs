use super::{column::RleColumn, pointermap::PointerColumn, range::RleRange, voxel::RgbVoxel};
use ndarray::{s, Array3, Axis};
use std::alloc::{alloc, dealloc, Layout};
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

impl Drop for RleVolume {
    fn drop(&mut self) {
        let num_pointers = (self.xsize * self.zsize) as usize;
        unsafe {
            dealloc(
                self.pointers as *mut u8,
                Layout::array::<PointerColumn>(num_pointers).unwrap(),
            );
            dealloc(
                self.columns,
                Layout::array::<u8>(self.columns_size as usize).unwrap(),
            );
        }
    }
}

impl RleVolume {
    /// Construct volume with no voxels with given size. `ysize` is up direction
    pub fn empty(xsize: usize, ysize: usize, zsize: usize) -> Self {
        // Restrict them to be power of two to allow easy mipmapping
        assert!(xsize == 0 || is_power2(xsize), "xsize is not power of two!");
        assert!(ysize == 0 || is_power2(ysize), "ysize is not power of two!");
        assert!(zsize == 0 || is_power2(zsize), "zsize is not power of two!");
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

/// Trick to check whether the number is power of two. Zero is counted as true.
fn is_power2(x: usize) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}

impl From<Array3<RgbVoxel>> for RleVolume {
    fn from(array: Array3<RgbVoxel>) -> Self {
        let (xsize, ysize, zsize) = array.dim();
        // Restrict them to be power of two to allow easy mipmapping
        assert!(xsize == 0 || is_power2(xsize), "xsize is not power of two!");
        assert!(ysize == 0 || is_power2(ysize), "ysize is not power of two!");
        assert!(zsize == 0 || is_power2(zsize), "zsize is not power of two!");
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
                    .slice(s![x..x + 1, .., z..z + 1])
                    .remove_axis(Axis(2))
                    .remove_axis(Axis(0));
                let rle_col = RleColumn::compress(&column.to_vec());
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
        let mut offset: usize = 0;
        unsafe {
            columns_array = alloc(Layout::array::<u8>(columns_offset).unwrap());
            for c in columns {
                let start = columns_array.offset(offset as isize);
                offset += c.pack_into(start);
            }
        }
        assert_eq!(columns_offset, offset, "Memory sizes should be equal");

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

impl Into<Array3<RgbVoxel>> for RleVolume {
    fn into(self) -> Array3<RgbVoxel> {
        let mut arr = Array3::zeros((
            self.xsize as usize,
            self.ysize as usize,
            self.zsize as usize,
        ));

        unsafe {
            let num_pointers = (self.xsize * self.zsize) as usize;
            for i in 0..num_pointers {
                let x = i % (self.xsize as usize);
                let z = i / (self.zsize as usize);
                let pcol = (*self.pointers.offset(i as isize)).clone();
                let col = RleColumn::unpack_from(
                    self.columns.offset(pcol.pointer as isize),
                    pcol.rle_count as usize,
                    Some(pcol.first_range),
                );

                for (y, color) in col.decompress().iter().enumerate() {
                    arr[(x, y, z)] = color.clone();
                }
            }
        }

        arr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr3;

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
    fn empty_zero_volume0() {
        let v = RleVolume::empty(0, 0, 0);
        let v = RleVolume::empty(1, 0, 0);
        let v = RleVolume::empty(0, 1, 0);
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

    fn encode_decode_array(voxels: Array3<RgbVoxel>, descr: &str) {
        let (x, y, z) = voxels.dim();
        let volume: RleVolume = voxels.clone().into();
        let decoded: Array3<RgbVoxel> = volume.into();
        assert_eq!(
            decoded, voxels,
            "Encoding-decoding {} volume {}x{}x{}",
            descr, x, y, z
        );
    }

    #[test]
    fn encode_array_test01() {
        let z = RgbVoxel::empty();
        let voxels: Array3<RgbVoxel> = arr3(&[[[z, z], [z, z]], [[z, z], [z, z]]]);
        encode_decode_array(voxels, "empty");
    }

    #[test]
    fn encode_array_test02() {
        let z = RgbVoxel::empty();
        let r = RgbVoxel::only_red(1);
        let voxels: Array3<RgbVoxel> = arr3(&[[[z, r], [z, r]], [[z, z], [z, z]]]);
        encode_decode_array(voxels, "simple");
    }
    #[test]
    fn encode_array_test03() {
        let r = RgbVoxel::only_red(1);
        let g = RgbVoxel::only_green(1);
        let voxels: Array3<RgbVoxel> = arr3(&[[[g, r], [g, r]], [[r, g], [g, r]]]);
        encode_decode_array(voxels, "filled");
    }

    #[test]
    fn encode_array_test04() {
        let z = RgbVoxel::empty();
        let r = RgbVoxel::only_red(1);
        let g = RgbVoxel::only_green(1);
        let voxels: Array3<RgbVoxel> = arr3(&[[[g, z], [g, r]], [[z, g], [g, r]]]);
        encode_decode_array(voxels, "partially filled");
    }

    #[test]
    fn encode_array_test05() {
        let z = RgbVoxel::empty();
        let r = RgbVoxel::only_red(1);
        let g = RgbVoxel::only_green(1);
        let voxels: Array3<RgbVoxel> = arr3(&[
            [[g, z, z, r], [g, r, z, z], [g, r, z, z], [g, r, z, z]],
            [[z, g, g, z], [g, r, r, g], [g, r, r, g], [g, r, r, g]],
            [[z, g, g, z], [g, r, r, g], [g, r, r, g], [g, r, r, g]],
            [[z, g, g, z], [g, r, r, g], [g, r, r, g], [g, r, r, g]],
        ]);
        encode_decode_array(voxels, "partially filled");
    }
}
