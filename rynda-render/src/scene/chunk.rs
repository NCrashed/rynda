use crate::math::{aabb::{AABB, HasBounding}, transform::{Transform, HasTransform}};
use rynda_format::types::volume::RleVolume;
use std::collections::HashMap;
use glam::IVec3;

/// Size of chunk in voxels in each dimension
pub const CHUNK_SIZE: usize = 256;

/// Size of single voxel in world units
pub const VOXEL_SIZE: f32 = 0.1;

/// A world object that consists of several RLE voxel chunks
pub struct ChunkedModel {
    /// Voxel chunks by their offsets
    pub volumes: HashMap<IVec3, RleVolume>,
    /// Relative transformation to the world coords
    pub transform: Transform,
    /// Minimum offset that a chunk has
    pub min_offset: IVec3,
    /// Maximum offset taht a chunk has
    pub max_offset: IVec3,
}

impl ChunkedModel {
    /// Create empty model
    pub fn new() -> Self {
        ChunkedModel {
            volumes: HashMap::new(),
            transform: Transform::new(),
            min_offset: IVec3::ZERO,
            max_offset: IVec3::ZERO,
        }
    }

    /// Insert new chunk at given coordinates
    pub fn add_chunk(&mut self, coords: IVec3, chunk: RleVolume) {
        self.volumes.insert(coords, chunk);
        self.update_boundary(coords);
    }

    fn update_boundary(&mut self, coord: IVec3) {
        if coord.x < self.min_offset.x {
            self.min_offset.x = coord.x;
        } else if coord.x > self.max_offset.x {
            self.max_offset.x = coord.x;
        }
        if coord.y < self.min_offset.y {
            self.min_offset.y = coord.y;
        } else if coord.y > self.max_offset.y {
            self.max_offset.y = coord.y;
        }
        if coord.z < self.min_offset.z {
            self.min_offset.z = coord.z;
        } else if coord.z > self.max_offset.z {
            self.max_offset.z = coord.z;
        }
    }
}

impl Default for ChunkedModel {
    fn default() -> Self {
        ChunkedModel::new()
    }
}

impl HasBounding for ChunkedModel {
    fn bounding_box(&self) -> AABB {
        if self.volumes.is_empty() {
            self.transform * AABB::new()
        } else {
            let size = VOXEL_SIZE * (CHUNK_SIZE as f32);

            let b = AABB {
                minv: self.min_offset.as_vec3() * size,
                maxv: (self.max_offset + IVec3::ONE).as_vec3()  * size,
            };
            self.transform * b
        }
    }
}

impl HasTransform for ChunkedModel {
    fn transformation(&self) -> Transform {
        self.transform
    }
}