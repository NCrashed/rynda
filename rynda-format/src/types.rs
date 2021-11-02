use modular_bitfield::specifiers::*;
use modular_bitfield::*;
use ndarray::{s, Array3, Axis};
use std::alloc::{alloc, Layout};
use std::{fmt, ptr};

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

/// Describes unpacked run length encoded column that is stored inside buffer in the `RleVolume`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RleColumn {
    /// Ranges of skipped-drawn voxels. May or may not contain first range depending of usage of the column.
    pub ranges: Vec<RleRange>,
    /// Color data that corresponds to drawn voxels in ranges vector.
    pub colors: Vec<RgbVoxel>,
}

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

#[repr(packed(2))]
#[derive(Clone, Copy)]
#[bitfield]
/// 16-bit RGB voxel with 5 bits for red and blue colors and 6 bits for green color. Human eye is considered
/// more sensitive to green tones. Zero values in all components are considered as an empty voxel.
pub struct RgbVoxel {
    pub red: B5,
    pub green: B6,
    pub blue: B5,
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

impl PartialEq for RgbVoxel {
    fn eq(&self, other: &Self) -> bool {
        self.red() == other.red() && self.green() == other.green() && self.blue() == other.blue()
    }
}

impl Eq for RgbVoxel {}

impl fmt::Debug for RgbVoxel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RgbVoxel")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

impl RleRange {
    /// Fast construction from skipped and drawn range
    pub fn range(skipped: u16, drawn: u8) -> Self {
        RleRange::new().with_skipped(skipped).with_drawn(drawn)
    }
}

impl RgbVoxel {
    /// Empty voxel is voxel with all values set to zero.
    pub fn empty() -> Self {
        RgbVoxel::from_bytes([0u8, 0])
    }

    /// Empty voxel is voxel with all values set to zero.
    pub fn is_empty(&self) -> bool {
        let bytes = self.into_bytes();
        bytes[0] == 0 && bytes[1] == 0
    }

    /// Shortcut for making color from red green blue bytes.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        RgbVoxel::new()
            .with_red(red)
            .with_green(green)
            .with_blue(blue)
    }

    /// Shortcut for making only red color shade
    pub fn only_red(red: u8) -> Self {
        Self::empty().with_red(red)
    }

    /// Shortcut for making only green color shade
    pub fn only_green(green: u8) -> Self {
        Self::empty().with_green(green)
    }

    /// Shortcut for making only blue color shade
    pub fn only_blue(blue: u8) -> Self {
        Self::empty().with_blue(blue)
    }
}

impl RleColumn {
    /// Given voxel column compact it into RLE column
    pub fn compress(array: &[RgbVoxel]) -> Self {
        let mut ranges = vec![];
        let mut colors = vec![];
        let mut skipped = 0;
        let mut drawn = 0;
        for color in array {
            if drawn == 0 {
                if skipped == 1023 {
                    ranges.push(RleRange::range(skipped, 0));
                    skipped = 0;
                }
                if color.is_empty() {
                    skipped += 1;
                } else {
                    drawn = 1;
                    colors.push(color.clone());
                }
            } else {
                if drawn == 63 {
                    ranges.push(RleRange::range(skipped, drawn));
                    skipped = 0;
                    drawn = 0;
                }

                if color.is_empty() {
                    ranges.push(RleRange::range(skipped, drawn));
                    skipped = 1;
                    drawn = 0;
                } else {
                    drawn += 1;
                    colors.push(color.clone());
                }
            }
        }
        if drawn > 0 || skipped > 0 {
            ranges.push(RleRange::range(skipped, drawn));
        }

        RleColumn { ranges, colors }
    }

    /// Convert the column to the raw column array of voxels
    pub fn decompress(&self) -> Vec<RgbVoxel> {
        unimplemented!()
    }

    /// Pack the data of the column inside the given memory chunk. It must by at least `memory_size` length. Returns the size written.
    pub fn pack_into(&self, mem: *mut u8) -> usize {
        unimplemented!()
    }

    /// Return amount of bytes the column will consume after packing
    pub fn memory_size(&self) -> usize {
        unimplemented!()
    }

    /// Return first range and rest column without that range but with it color data
    pub fn split_head(self) -> (RleRange, Self) {
        unimplemented!()
    }

    /// Return count of RLE intervals in that column
    pub fn intervals_count(&self) -> usize {
        unimplemented!()
    }
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
                let (first_range, rest_column) = rle_col.split_head();
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
    fn color_tests() {
        assert!(RgbVoxel::empty().is_empty(), "Empty voxel is not empty");
        assert_eq!(
            RgbVoxel::rgb(31, 0, 0).into_bytes(),
            [0b00011111, 0b00000000],
            "Red bits are not in expected place"
        );
        assert_eq!(
            RgbVoxel::rgb(0, 63, 0).into_bytes(),
            [0b11100000, 0b00000111],
            "Green bits are not in expected place"
        );
        assert_eq!(
            RgbVoxel::rgb(0, 0, 31).into_bytes(),
            [0b00000000, 0b11111000],
            "Green bits are not in expected place"
        );
    }

    #[test]
    fn column_tests() {
        assert_eq!(
            RleColumn::compress(&[]),
            RleColumn {
                ranges: vec![],
                colors: vec![],
            },
            "Compression of empty column"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty()]),
            RleColumn {
                ranges: vec![RleRange::range(1, 0)],
                colors: vec![],
            },
            "Compression of single empty voxel"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(); 5]),
            RleColumn {
                ranges: vec![RleRange::range(5, 0)],
                colors: vec![],
            },
            "Compression of empty column"
        );
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::empty(), RgbVoxel::only_red(1)]),
            RleColumn {
                ranges: vec![RleRange::range(2, 1)],
                colors: vec![RgbVoxel::only_red(1)],
            },
            "Compression of simple column"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::only_green(1)]),
            RleColumn {
                ranges: vec![RleRange::range(0, 2)],
                colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_green(1)],
            },
            "Compression of two non empty voxels"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::only_red(1); 64]),
            RleColumn {
                ranges: vec![RleRange::range(0, 63), RleRange::range(0, 1)],
                colors: vec![RgbVoxel::only_red(1); 64],
            },
            "Compression of column with drawn overflow"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(); 1024]),
            RleColumn {
                ranges: vec![RleRange::range(1023, 0), RleRange::range(1, 0)],
                colors: vec![],
            },
            "Compression of column with skipped overflow"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty(), RgbVoxel::only_blue(1)]),
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 1)],
                colors: vec![ RgbVoxel::only_red(1),  RgbVoxel::only_blue(1)],
            },
            "Compression of column with two ranges"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()]),
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 0)],
                colors: vec![ RgbVoxel::only_red(1)],
            },
            "Compression of column with two ranges, second empty"
        );
    }

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
