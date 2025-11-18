use std::f64::consts::TAU;
use crate::core::vec3::Vec3;

/// Pose de cámara simple para órbita + zoom (rotación y distancia)
#[derive(Clone, Copy)]
pub struct CameraPose {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_deg: f64,
}

pub struct CameraOrbit {
    pub center: Vec3,
    pub base_radius: f64,
    pub zoom_amp: f64,
    pub height: f64,
}

impl CameraOrbit {
    pub fn new(center: Vec3) -> Self {
        Self {
            center,
            base_radius: 18.0,
            zoom_amp: 2.0,
            height: 8.0,
        }
    }

    /// t en segundos; una vuelta ~10s (ajústalo a tu gusto)
    pub fn pose_at(&self, t: f64) -> CameraPose {
        let phase = (t / 10.0) * TAU;
        let radius = self.base_radius + self.zoom_amp * (2.0 * phase).sin();
        let eye = Vec3::new(
            self.center.x + radius * phase.cos(),
            self.height,
            self.center.z + radius * phase.sin(),
        );
        CameraPose {
            eye,
            target: self.center,
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_deg: 60.0,
        }
    }
}
