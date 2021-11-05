use super::{
    range::{RleRange, RLE_RANGE_SIZE, RLE_SKIPPED_MAX, RLE_DRAWN_MAX},
    voxel::{RgbVoxel, RGB_VOXEL_SIZE},
};

/// Describes unpacked run length encoded column that is stored inside buffer in the `RleVolume`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RleColumn {
    /// Ranges of skipped-drawn voxels. May or may not contain first range depending of usage of the column.
    pub ranges: Vec<RleRange>,
    /// Color data that corresponds to drawn voxels in ranges vector.
    pub colors: Vec<RgbVoxel>,
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
        let mut voxels = vec![];
        let mut col_offset = 0;

        for range in self.ranges.iter() {
            for _ in 0..range.skipped() {
                voxels.push(RgbVoxel::empty());
            }
            for _ in 0..range.drawn() {
                voxels.push(self.colors[col_offset]);
                col_offset += 1;
            }
        }

        voxels
    }

    /// Pack the data of the column inside the given memory chunk. It must by at least `memory_size` length. Returns the size written.
    pub unsafe fn pack_into(&self, mem: *mut u8) -> usize {
        let mut offset: usize = 0;
        for range in self.ranges.iter() {
            let ptr = mem.offset(offset as isize);
            let range_bytes = range.into_bytes();
            let range_len = range_bytes.len();
            ptr.copy_from_nonoverlapping(range_bytes.as_ptr(), range_len);
            offset += range_len;
        }
        for color in self.colors.iter() {
            let ptr = mem.offset(offset as isize);
            let color_bytes = color.into_bytes();
            let color_len = color_bytes.len();
            ptr.copy_from_nonoverlapping(color_bytes.as_ptr(), color_len);
            offset += color_len;
        }
        offset
    }

    /// Unpack the data of the column from raw memory chunk. It must contain all the column which size depends on rle_count of ranges.
    pub unsafe fn unpack_from(mem: *const u8, rle_count: usize, first_range: Option<RleRange>) -> Self {
        let mut ranges;
        let mut colors = vec![];
        let mut drawn;
        
        match first_range {
            None => {
                ranges = vec![];
                drawn = 0;
            }
            Some(r) => {
                ranges = vec![r];
                drawn = r.drawn();
            }
        };

        for i in 0 .. rle_count {
            let ptr = mem.offset((i*RLE_RANGE_SIZE) as isize);
            let mut range_bytes = [0; 2];
            range_bytes.as_mut_ptr().copy_from_nonoverlapping(ptr, range_bytes.len());
            let range = RleRange::from_bytes(range_bytes);
            drawn += range.drawn();
            ranges.push(range);
        }
        for i in 0 .. drawn {
            let ptr = mem.offset((rle_count*RLE_RANGE_SIZE + (i as usize)*RGB_VOXEL_SIZE) as isize);
            let mut color_bytes = [0; 2];
            color_bytes.as_mut_ptr().copy_from_nonoverlapping(ptr, color_bytes.len());
            let color = RgbVoxel::from_bytes(color_bytes);
            colors.push(color);
        }

        RleColumn { ranges, colors }
    }

    /// Return amount of bytes the column will consume after packing
    pub fn memory_size(&self) -> usize {
        self.ranges.len() * RLE_RANGE_SIZE + self.colors.len() * RGB_VOXEL_SIZE
    }

    /// Return first range and rest column without that range but with it color data
    pub fn split_head(mut self) -> Option<(RleRange, Self)> {
        if self.ranges.len() == 0 {
            return None;
        }

        let first = self.ranges.drain(0..1).next().unwrap();
        Some((first, self))
    }

    /// Return count of RLE intervals in that column
    pub fn intervals_count(&self) -> usize {
        self.ranges.len()
    }

    /// Merge repeated empty ranges and merge drawn ranges. 
    pub fn optimize(self) -> Self {
        let mut new_ranges = vec![];
        let mut mprev_range: Option<RleRange> = None;

        for range in self.ranges {
            match mprev_range {
                None => mprev_range = Some(range),
                Some(mut prev_range) => {
                    let new_skipped = (prev_range.skipped() as usize) + (range.skipped() as usize);
                    let new_drawn = (prev_range.drawn() as usize) + (range.drawn() as usize);
                    if prev_range.drawn() == 0 && new_skipped < RLE_SKIPPED_MAX {
                        prev_range.set_skipped(new_skipped as u16);
                        prev_range.set_drawn(range.drawn());
                        mprev_range = Some(prev_range);
                    } else if range.skipped() == 0 && new_drawn < RLE_DRAWN_MAX {
                        prev_range.set_drawn(new_drawn as u8);
                        mprev_range = Some(prev_range);
                    } else {
                        new_ranges.push(prev_range);
                        mprev_range = Some(range);
                    }
                }
            }
        }
        match mprev_range {
            None => (),
            Some(prev_range) => new_ranges.push(prev_range),
        }

        RleColumn { ranges: new_ranges, colors: self.colors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_compress_tests() {
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
            RleColumn::compress(&[
                RgbVoxel::empty(),
                RgbVoxel::only_red(1),
                RgbVoxel::empty(),
                RgbVoxel::only_blue(1)
            ]),
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
            },
            "Compression of column with two ranges"
        );

        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()]),
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 0)],
                colors: vec![RgbVoxel::only_red(1)],
            },
            "Compression of column with two ranges, second empty"
        );
    }

    #[test]
    fn column_decompress_test() {
        assert_eq!(
            RleColumn {
                ranges: vec![],
                colors: vec![],
            }
            .decompress(),
            vec![],
            "Decompression of empty column"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(1, 0)],
                colors: vec![],
            }
            .decompress(),
            vec![RgbVoxel::empty()],
            "Decompression of single empty voxel"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(5, 0)],
                colors: vec![],
            }
            .decompress(),
            vec![RgbVoxel::empty(); 5],
            "Decompression of empty column"
        );
        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(2, 1)],
                colors: vec![RgbVoxel::only_red(1)],
            }
            .decompress(),
            vec![RgbVoxel::empty(), RgbVoxel::empty(), RgbVoxel::only_red(1)],
            "Decompression of simple column"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(0, 2)],
                colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_green(1)],
            }
            .decompress(),
            vec![RgbVoxel::only_red(1), RgbVoxel::only_green(1)],
            "Decompression of two non empty voxels"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(0, 63), RleRange::range(0, 1)],
                colors: vec![RgbVoxel::only_red(1); 64],
            }
            .decompress(),
            vec![RgbVoxel::only_red(1); 64],
            "Decompression of column with drawn overflow"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(1023, 0), RleRange::range(1, 0)],
                colors: vec![],
            }
            .decompress(),
            vec![RgbVoxel::empty(); 1024],
            "Decompression of column with skipped overflow"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
            }
            .decompress(),
            vec![
                RgbVoxel::empty(),
                RgbVoxel::only_red(1),
                RgbVoxel::empty(),
                RgbVoxel::only_blue(1)
            ],
            "Decompression of column with two ranges"
        );

        assert_eq!(
            RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 0)],
                colors: vec![RgbVoxel::only_red(1)],
            }
            .decompress(),
            vec![RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()],
            "Decompression of column with two ranges, second empty"
        );
    }

    #[test]
    fn split_head_test() {
        assert_eq!(
            RleColumn::compress(&[]).split_head(),
            None,
            "Splitting empty column produces non empty result"
        );
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty()]).split_head(),
            Some((
                RleRange::range(1, 0),
                RleColumn {
                    ranges: vec![],
                    colors: vec![]
                }
            )),
            "Splitting column with single range produces wrong result"
        );
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::empty()]).split_head(),
            Some((
                RleRange::range(0, 1),
                RleColumn {
                    ranges: vec![RleRange::range(1, 0)],
                    colors: vec![RgbVoxel::only_red(1)]
                }
            )),
            "Splitting column with two ranges produces wrong result"
        );
    }

    #[test]
    fn intervals_count_test() {
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()])
                .intervals_count(),
            2
        );
    }

    #[test]
    fn optimize_test_01() {
        let column_a = RleColumn {
                ranges: vec![],
                colors: vec![],
            };
        let column_b = RleColumn {
            ranges: vec![],
            colors: vec![],
        };
        assert_eq!(column_a.optimize(), column_b, "Empty column optmizes to empty column");
    }

    #[test]
    fn optimize_test_02() {
        let column_a = RleColumn {
                ranges: vec![RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_red(1)],
            };
        let column_b = RleColumn {
            ranges: vec![RleRange::range(1, 1)],
            colors: vec![RgbVoxel::only_red(1)],
        };
        assert_eq!(column_a.optimize(), column_b, "Single sized column optmizes to the same column");
    }

    #[test]
    fn optimize_test_03() {
        let column_a = RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
            };
        let column_b = RleColumn {
            ranges: vec![RleRange::range(1, 1), RleRange::range(1, 1)],
            colors: vec![RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
        };
        assert_eq!(column_a.optimize(), column_b, "Optimized column optmizes to the same column");
    }

    #[test]
    fn optimize_test_04() {
        let column_a = RleColumn {
                ranges: vec![RleRange::range(1, 0), RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_blue(1)],
            };
        let column_b = RleColumn {
            ranges: vec![RleRange::range(2, 1)],
            colors: vec![RgbVoxel::only_blue(1)],
        };
        assert_eq!(column_a.optimize(), column_b, "Skipped range optimization");
    }

    #[test]
    fn optimize_test_05() {
        let column_a = RleColumn {
                ranges: vec![RleRange::range(1, 0), RleRange::range(1, 0), RleRange::range(1, 1), RleRange::range(1, 0)],
                colors: vec![RgbVoxel::only_blue(1)],
            };
        let column_b = RleColumn {
            ranges: vec![RleRange::range(3, 1), RleRange::range(1, 0)],
            colors: vec![RgbVoxel::only_blue(1)],
        };
        assert_eq!(column_a.optimize(), column_b, "Skipped range optimization");
    }

    #[test]
    fn optimize_test_06() {
        let column_a = RleColumn {
                ranges: vec![RleRange::range(1, 1), RleRange::range(0, 1), RleRange::range(1, 1)],
                colors: vec![RgbVoxel::only_blue(1), RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
            };
        let column_b = RleColumn {
            ranges: vec![RleRange::range(1, 2), RleRange::range(1, 1)],
            colors: vec![RgbVoxel::only_blue(1), RgbVoxel::only_red(1), RgbVoxel::only_blue(1)],
        };
        assert_eq!(column_a.optimize(), column_b, "Drawn range optimization");
    }

    #[test]
    fn pack_into_test_empty() {
        let column = RleColumn::compress(&[]);
        let mut buffer = vec![];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(buffer, vec![], "Packing column with no ranges");
    }

    #[test]
    fn pack_into_test_simple01() {
        let column = RleColumn::compress(&[RgbVoxel::empty()]);
        let mut buffer = vec![0; 2];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(buffer, vec![1, 0], "Packing column with single empty range");
    }

    #[test]
    fn pack_into_test_simple02() {
        let column = RleColumn::compress(&[RgbVoxel::only_green(1)]);
        let mut buffer = vec![0; 4];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(
            buffer,
            vec![0, 0b00000100, 0b00100000, 0],
            "Packing column with single color range"
        );
    }

    #[test]
    fn pack_into_test_simple03() {
        let column = RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::only_red(1)]);
        let mut buffer = vec![0; 6];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(
            buffer,
            vec![0, 0b00001000, 0b00000001, 0, 0b00000001, 0],
            "Packing column with single color range"
        );
    }

    #[test]
    fn pack_into_test_simple04() {
        let (_, column) = RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::only_red(1)]).split_head().unwrap();
        let mut buffer = vec![0; 4];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(
            buffer,
            vec![0b00000001, 0, 0b00000001, 0],
            "Packing column with single color range with split"
        );
    }


    #[test]
    fn pack_into_test_complex() {
        let column = RleColumn::compress(&[
            RgbVoxel::only_red(1),
            RgbVoxel::empty(),
            RgbVoxel::only_blue(1),
        ]);
        let mut buffer = vec![0; 8];
        let size;
        unsafe {
            size = column.pack_into(buffer.as_mut_ptr());
        }
        assert_eq!(size, buffer.len(), "Packed size is not equal buffer size");
        assert_eq!(
            buffer,
            vec![0, 0b00000100, 1, 0b00000100, 0b00000001, 0, 0, 0b00001000],
            "Packing column with two ranges"
        );
    }

    #[test]
    fn unpack_from_test_empty() {
        let mut buffer = vec![];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 0, None);
        }
        assert_eq!(column, RleColumn::compress(&[]), "Unpacking column with no ranges");
    }

    #[test]
    fn unpack_from_test_empty_first() {
        let mut buffer = vec![];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 0, Some(RleRange::range(1, 0)));
        }
        assert_eq!(column, RleColumn::compress(&[]), "Unpacking column with no ranges");
    }

    #[test]
    fn unpack_from_test_simple01() {
        let mut buffer = vec![1, 0];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 1, None);
        }
        assert_eq!(column, RleColumn::compress(&[RgbVoxel::empty()]), "Unpacking column with single range");
    }

    #[test]
    fn unpack_from_test_simple01_first() {
        let mut buffer = vec![1, 0];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 1, Some(RleRange::range(1, 0)));
        }
        assert_eq!(column.optimize(), RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::empty()]), "Unpacking column with single range");
    }

    #[test]
    fn unpack_from_test_simple02() {
        let mut buffer = vec![0, 0b00000100, 0b00100000, 0];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 1, None);
        }
        assert_eq!(column, RleColumn::compress(&[RgbVoxel::only_green(1)]), "Unpacking column with single range");
    }

    #[test]
    fn unpack_from_test_simple03() {
        let mut buffer = vec![0b00000001, 0, 0b00000001, 0];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 0, Some(RleRange::range(0, 2)));
        }
        assert_eq!(column, RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::only_red(1)]), "Unpacking column with single range");
    }

    #[test]
    fn unpack_from_test_simple04() {
        let mut buffer = vec![1, 0b00001000, 0b00000001, 0, 0b00000001, 0, 0b00100000, 0, 0b00100000, 0];
        let column;

        unsafe {
            column = RleColumn::unpack_from(buffer.as_mut_ptr(), 1, Some(RleRange::range(0, 2)));
        }
        assert_eq!(column, RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::only_red(1), RgbVoxel::empty(), RgbVoxel::only_green(1), RgbVoxel::only_green(1)]), "Unpacking column with two range");
    }

    #[test]
    fn memory_size_test() {
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()])
                .memory_size(),
            6
        );

        let column =
            RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()]);
        let mut buffer = vec![0; 1024];
        unsafe {
            assert_eq!(column.pack_into(buffer.as_mut_ptr()), column.memory_size());
        }
    }
}
