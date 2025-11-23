use crate::core::math::Vec3;

pub mod block;
pub mod builder;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub albedo: Vec3,
    pub texture_path: Option<&'static str>,
    pub uv_scale: f64,
    pub specular: f64,
    pub reflection: f64,
    pub emissive: Vec3,
    pub animated: bool,
}

impl Material {
    pub fn new(name: &str, albedo: Vec3, texture_path: Option<&'static str>) -> Self {
        Self {
            name: name.to_string(),
            albedo,
            texture_path,
            uv_scale: 1.0,
            specular: 0.0,
            reflection: 0.0,
            emissive: Vec3::new(0.0, 0.0, 0.0),
            animated: false,
        }
    }

    pub fn with_uv_scale(mut self, s: f64) -> Self {
        self.uv_scale = s;
        self
    }

    pub fn with_specular(mut self, s: f64) -> Self {
        self.specular = s;
        self
    }

    pub fn with_reflection(mut self, r: f64) -> Self {
        self.reflection = r;
        self
    }

    pub fn with_emissive(mut self, e: Vec3) -> Self {
        self.emissive = e;
        self
    }

    pub fn animated(mut self, flag: bool) -> Self {
        self.animated = flag;
        self
    }
}

#[derive(Clone)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub voxels: Vec<block::Voxel>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
            voxels: Vec::new(),
        }
    }
}
