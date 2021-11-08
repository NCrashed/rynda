use super::types::volume::RleVolume;
use super::types::voxel::RgbVoxel;
use dot_vox::{self, DotVoxData};
use ndarray::Array3;

/// Convert color (LE-encoded into i32) with 8-bit channels into 5-6-5-channel one.
fn shakal(rgb: u32) -> RgbVoxel {
    let red = ((rgb & 0xFF) as u8) >> 3;
    let green = (((rgb >> 8) & 0xFF) as u8) >> 2;
    let blue = (((rgb >> 16) & 0xFF) as u8) >> 3;
    RgbVoxel::rgb(red, green, blue)
}

pub fn vox_to_rle_volume(filename: &str) -> Result<RleVolume, &str> {
    Ok(dot_vox::load(filename)?.into())
}

impl From<DotVoxData> for RleVolume {
    fn from(data: DotVoxData) -> Self {
        let model = &data.models[0];
        let xsize = model.size.x as usize;
        let ysize = model.size.y as usize;
        let zsize = model.size.z as usize;
        let mut space = Array3::from_shape_vec(
            (xsize, zsize, ysize),
            vec![RgbVoxel::empty(); xsize * ysize * zsize],
        )
        .unwrap();

        for voxel in model.voxels.iter() {
            space[(voxel.x as usize, voxel.z as usize, voxel.y as usize)] =
                shakal(data.palette[voxel.i as usize]);
        }

        RleVolume::from(space)
    }
}
