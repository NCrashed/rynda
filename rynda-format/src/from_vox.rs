mod types;
use std::{collections::HashMap};

use dot_vox::{self, Voxel};
use modular_bitfield::{
    bitfield,
    prelude::{B8},
};
use rynda_format::types::voxel::RgbVoxel;
use rynda_format::types::range::RleRange;

/// Uncompressed, 32-bit color.
#[bitfield]
struct RGB32 {
    pub red: B8,
    pub green: B8,
    pub blue: B8,
    pub alpha: B8,
}

/// Convert a vector of colors into deep-RLE pair of draw/skips and colors.
/// Considers (0, 0, 0)-color as void.
fn deep_rle_compress(input: &Vec<RgbVoxel>) -> (Vec<RleRange>, Vec<RgbVoxel>) {

    /// Compression automata.
    enum CompressState {
        Start,
        VoidFrom(u8),
        VoidAndFillFrom(u8, u8, RgbVoxel),
    }
    use CompressState::*;

    let mut rles = vec![];
    let mut voxels = vec![];
    let last = (input.len() - 1) as u8;

    let end = input
        .iter()
        .zip(0..=255_u8)
        .fold(Start, |state, (voxel, index)| match (state, voxel) {

            // Start cases.
            (Start, rgb) if is_empty(rgb) => VoidFrom(0),
            (Start, rgb) => VoidAndFillFrom(0, 0, *rgb),

            // Going across void.
            (VoidFrom(start), rgb) if is_empty(rgb) => VoidFrom(start),

            // Void-fill border.
            (VoidFrom(start), rgb) => VoidAndFillFrom(start, index, *rgb),

            // There are only 6 bits for drawing, so we restart a range here.
            (VoidAndFillFrom(empty, fill, rgb), rgb1) if index - fill == 63 => {
                rles.push(make_rle_range(fill, empty, index));
                voxels.push(rgb);
                VoidAndFillFrom(index, index, *rgb1)
            }

            // Fill-void border.
            (VoidAndFillFrom(empty, fill, rgb), rgb1) if is_empty(rgb1) => {
                rles.push(make_rle_range(fill, empty, index));
                voxels.push(rgb);
                VoidFrom(index)
            }

            // Going across fill.
            (VoidAndFillFrom(empty, fill, rgb), rgb1) if *rgb1 == rgb => {
                VoidAndFillFrom(empty, fill, rgb)
            }

            // Fill-fill1 border.
            (VoidAndFillFrom(empty, fill, rgb), rgb1) => {
                rles.push(make_rle_range(fill, empty, index));
                voxels.push(rgb);
                VoidAndFillFrom(index, index, *rgb1)
            }
        });

    // Closing the last range.
    if let VoidAndFillFrom(empty, fill, rgb) = end {
        rles.push(make_rle_range(fill, empty, last));
        voxels.push(rgb);
    }

    // If the line was all void, add "all void" range and phantom black color,
    // so we don't need complex self-checks later - each range has 1 color.
    if rles.is_empty() {
        rles.push(make_rle_range(last, 0, last));
        voxels.push(RgbVoxel::rgb(0, 0, 0))
    }
    (rles, voxels)
}

/// Decompress a pair (ranges, voxels) into a vector of voxels.
fn deep_rle_decompress(rles: &Vec<RleRange>, voxels: &Vec<RgbVoxel>) -> Vec<RgbVoxel> {
    if rles.len() != voxels.len() {
        panic!("decompress: rles.len() != voxels.len()")
    }

    let mut res: Vec<RgbVoxel> = rles
        .iter()
        .zip(voxels)
        .flat_map(|(range, voxel)| {
            cat(
                // Skip.
                vec![RgbVoxel::rgb(0, 0, 0); range.skipped() as usize],
                // Draw.
                vec![*voxel; range.drawn() as usize],
            )
        })
        .collect();

    // Make sure the result is of correct length.
    res.resize(256, RgbVoxel::rgb(0, 0, 0));
    res
}

/// ._.
fn cat<A: Copy>(a: Vec<A>, b: Vec<A>) -> Vec<A> {
    let mut r = Vec::<A>::with_capacity(a.len() + b.len());
    r.extend(a);
    r.extend(b);
    r
}

/// Check if the color is void.
fn is_empty(rgb: &RgbVoxel) -> bool {
    rgb.into_bytes() == [0, 0]
}

/// Construct an RLE range.
fn make_rle_range(fill: u8, empty: u8, index: u8) -> RleRange {
    let mut range = RleRange::new();
    range.set_drawn(index - fill);
    range.set_skipped((fill - empty).into());
    range
}

/// Convert color (LE-encoded into i32) with 8-bit channels into 5-6-5-channel one.
fn shakal(rgb: u32) -> RgbVoxel {
    let rgb32 = RGB32::from_bytes(rgb.to_le_bytes());
    RgbVoxel::rgb(rgb32.red() >> 3, rgb32.green() >> 2, rgb32.blue() >> 3)
}

/// Self-test on an example asset.
fn main() -> Result<(), String> {
    let vox = dot_vox::load("assets/test_model.vox")?;
    let mut voxel_map = HashMap::<(u8, u8, u8), RgbVoxel>::from([]);
    println!("Start");
    for model in &vox.models {
        for Voxel { x, y, z, i } in &model.voxels {
            voxel_map.insert((*x, *y, *z), shakal(vox.palette[*i as usize]));
        }
    }
    println!("Map done");
    for x in 0..=255 {
        for z in 0..=255 {
            let line = make_xz_normal(&voxel_map, x, z);
            let (rles, voxels) = deep_rle_compress(&line);
            let back = deep_rle_decompress(&rles, &voxels);
            assert_vec_equal(&back, &line)
        }
    }
    println!("Decompressed");
    return Ok(());
}

fn make_xz_normal(voxel_map: &HashMap<(u8, u8, u8), RgbVoxel>, x: u8, z: u8) -> Vec<RgbVoxel> {
    let mut line = vec![];
    for y in 0..=255 {
        line.push(*voxel_map.get(&(x, y, z)).unwrap_or(&RgbVoxel::new()))
    }
    line
}

fn assert_vec_equal(back: &Vec<RgbVoxel>, line: &Vec<RgbVoxel>) {
    for i in 0..256 {
        if back[i] != line[i] {
            println!("");
            println!("[{:?}]", i);
            println!("{:?}", line[i]);
            println!("!=");
            println!("{:?}", back[i]);
            panic!("decompress . compress =/= id");
        }
    }
}
