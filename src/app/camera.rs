use crate::core::math::Vec3;

#[derive(Clone, Copy)]
pub struct CameraPose {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_deg: f64,
}

pub struct CameraController {
    center: Vec3,
    radius: f64,
    height: f64,
    yaw: f64,
}

impl CameraController {
    pub fn new(center: Vec3) -> Self {
        Self {
            center,
            radius: 18.0,
            height: 8.0,
            yaw: 0.0,
        }
    }

    pub fn apply_input(
        &mut self,
        left: bool,
        right: bool,
        zoom_in: bool,
        zoom_out: bool,
        height_up: bool,
        height_down: bool,
        dt: f64,
    ) {
        let rot_speed = 1.5;
        let zoom_speed = 10.0;
        let height_speed = 10.0;

        if left {
            self.yaw -= rot_speed * dt;
        }
        if right {
            self.yaw += rot_speed * dt;
        }
        if zoom_in {
            self.radius -= zoom_speed * dt;
        }
        if zoom_out {
            self.radius += zoom_speed * dt;
        }
        if height_up {
            self.height += height_speed * dt;
        }
        if height_down {
            self.height -= height_speed * dt;
        }

        if self.radius < 6.0 {
            self.radius = 6.0;
        }
        if self.radius > 40.0 {
            self.radius = 40.0;
        }
        if self.height < 2.0 {
            self.height = 2.0;
        }
        if self.height > 20.0 {
            self.height = 20.0;
        }
    }

    pub fn pose(&self) -> CameraPose {
        let eye = Vec3::new(
            self.center.x + self.radius * self.yaw.cos(),
            self.height,
            self.center.z + self.radius * self.yaw.sin(),
        );
        CameraPose {
            eye,
            target: self.center,
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_deg: 60.0,
        }
    }
}
