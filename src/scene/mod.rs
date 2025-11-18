use crate::core::vec3::Vec3;

pub mod mesh;
pub mod voxel;
pub mod builder;

// Re-export para que main.rs pueda seguir usando build_minecraft_house_scene()
pub use builder::build_minecraft_house_scene;

/* ========================= Material ========================= */

#[derive(Clone)]
pub struct Material {
    pub name: &'static str,

    /// Color base cuando no hay textura (o se multiplica con la textura)
    pub albedo: Vec3,

    /// Fuerza especular (0..1)
    pub specular: f64,

    /// Transparencia (0..1) – para refracción si la activas
    pub transparency: f64,

    /// Reflectividad (0..1) – para reflexión si la activas
    pub reflectivity: f64,

    /// Índice de refracción (vidrio ~1.5)
    pub ior: f64,

    /// Emisión (antorchas, campfires, etc.)
    pub emissive: Vec3,

    /// Ruta a textura BMP (24 bpp). Si None, usa solo albedo.
    pub texture_path: Option<&'static str>,

    /// Escala de UV por material (cómo de “repetida” se ve la textura).
    /// 1.0 = 1 tile por unidad, 4.0 = 4 tiles por unidad, etc.
    pub uv_scale: f64,

    /// Si true, aplicará animación simple a las UV (agua, lava, etc.)
    pub animated_uv: bool,
}

impl Material {
    pub fn new(
        name: &'static str,
        albedo: Vec3,
        texture_path: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            albedo,
            specular: 0.04,
            transparency: 0.0,
            reflectivity: 0.0,
            ior: 1.5,
            emissive: Vec3::new(0.0, 0.0, 0.0),
            texture_path,
            uv_scale: 1.0,
            animated_uv: false,
        }
    }

    pub fn with_uv_scale(mut self, s: f64) -> Self { self.uv_scale = s; self }
    pub fn with_specular(mut self, k: f64) -> Self { self.specular = k; self }
    pub fn with_emissive(mut self, e: Vec3) -> Self { self.emissive = e; self }
    pub fn animated(mut self, on: bool) -> Self { self.animated_uv = on; self }
    pub fn with_reflection(mut self, r: f64) -> Self { self.reflectivity = r; self }
    pub fn with_transparency(mut self, t: f64, ior: f64) -> Self { self.transparency = t; self.ior = ior; self }
}

/* ========================= Skybox ========================= */

#[derive(Clone, Default)]
pub struct Skybox {
    pub right:  Option<&'static str>, // +X
    pub left:   Option<&'static str>, // -X
    pub top:    Option<&'static str>, // +Y
    pub bottom: Option<&'static str>, // -Y
    pub front:  Option<&'static str>, // +Z
    pub back:   Option<&'static str>, // -Z
}

/* ========================= Portales ========================= */

#[derive(Clone)]
pub struct Portal {
    pub min: Vec3,
    pub max: Vec3,
    /// Centro destino al que “aparece” el rayo
    pub to_pos: Vec3,
    /// Rotación Y (grados) aplicada a la dirección del rayo al salir
    pub rot_y_deg: f64,
}

/* ========================= Scene ========================= */

#[derive(Clone)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub voxels: Vec<voxel::Voxel>,
    pub triangles: Vec<mesh::Tri>,
    pub skybox: Skybox,
    pub portals: Vec<Portal>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
            voxels: Vec::new(),
            triangles: Vec::new(),
            skybox: Skybox::default(),
            portals: Vec::new(),
        }
    }

    pub fn new_empty() -> Self { Self::new() }
}
