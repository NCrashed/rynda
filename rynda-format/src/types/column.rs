use super::{range::{RleRange, RLE_RANGE_SIZE}, voxel::{RgbVoxel, RGB_VOXEL_SIZE}};

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
    pub fn pack_into(&self, mem: *mut u8) -> usize {
        unimplemented!()
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
            Some((RleRange::range(1, 0), RleColumn {
                ranges: vec![],
                colors: vec![]
            })),
            "Splitting column with single range produces wrong result"
        );
        assert_eq!(
            RleColumn::compress(&[RgbVoxel::only_red(1), RgbVoxel::empty()]).split_head(),
            Some((RleRange::range(0, 1), RleColumn {
                ranges: vec![RleRange::range(1, 0)],
                colors: vec![RgbVoxel::only_red(1)]
            })),
            "Splitting column with two ranges produces wrong result"
        );
    }

    #[test]
    fn intervals_count_test() {
        assert_eq!(RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()]).intervals_count(), 2);
    }

    #[test]
    fn memory_size_test() {
        assert_eq!(RleColumn::compress(&[RgbVoxel::empty(), RgbVoxel::only_red(1), RgbVoxel::empty()]).memory_size(), 6);
    }
}
