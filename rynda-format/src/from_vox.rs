
use std::{collections::HashMap};

use dot_vox::{self, DotVoxData, Voxel};
use modular_bitfield::{
    bitfield,
    prelude::{B8},
};
use ndarray::{Array3, arr3, array};
use super::types::volume::RleVolume;
use super::types::voxel::RgbVoxel;
use super::types::range::RleRange;

/// Uncompressed, 32-bit color.
#[derive(Clone, Copy)]
#[bitfield]
struct RGB32 {
    pub red: B8,
    pub green: B8,
    pub blue: B8,
    pub alpha: B8,
}

/// Convert color (LE-encoded into i32) with 8-bit channels into 5-6-5-channel one.
fn shakal(rgb: u32) -> RgbVoxel {
    let rgb32 = RGB32::from_bytes(rgb.to_le_bytes());
    RgbVoxel::rgb(rgb32.red() >> 3, rgb32.green() >> 2, rgb32.blue() >> 3)
}

pub fn vox_to_rle_volume(filename: &str) -> Result<RleVolume, &str> {
    let data = dot_vox::load(filename)?;

    let mut space = vec![RgbVoxel::empty(); 256 * 256 * 256];
    for voxel in &data.models[0].voxels {
        space[metric(voxel.x as usize, voxel.y as usize, voxel.z as usize)] =
            shakal(data.palette[voxel.i as usize]);
    }

    Ok(RleVolume::from(Array3::from_shape_fn((256, 256, 256), |(x, y, z)| {
        space[metric(x, y, z)]
    })))
}

fn metric(x: usize, y: usize, z: usize) -> usize {
    (x * 256 + y) * 256 + z
}
