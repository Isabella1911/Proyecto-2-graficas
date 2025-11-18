use crate::core::vec3::Vec3;

/// Voxel axis-aligned (cubo unidad), definido por celda (i,j,k) y material.
/// Aquí guardamos el AABB en espacio mundo para facilitar intersecciones.
#[derive(Clone)]
pub struct Voxel {
    pub min: Vec3,
    pub max: Vec3,
    pub mat_id: usize,
}

impl Voxel {
    /// Crea un voxel de 1x1x1 en coordenadas de grilla (enteras)
    pub fn from_grid(i: usize, j: usize, k: usize, mat_id: usize) -> Self {
        let (x0,y0,z0) = (i as f64, j as f64, k as f64);
        let min = Vec3::new(x0, y0, z0);
        let max = Vec3::new(x0+1.0, y0+1.0, z0+1.0);
        Self { min, max, mat_id }
    }
}
