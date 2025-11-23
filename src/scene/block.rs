
use crate::core::math::Vec3;


#[derive(Copy, Clone)]
pub struct Voxel {
    pub min: Vec3,
    pub max: Vec3,
    pub mat_id: usize,
}
