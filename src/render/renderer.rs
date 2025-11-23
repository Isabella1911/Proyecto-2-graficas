use crate::app::camera::CameraPose;
use crate::app::daynight::DayNight;
use crate::core::math::{Color, Vec3, clamp01};
use crate::core::ray::Ray;
use crate::framebuffer::Framebuffer;
use crate::scene::Scene;
use crate::scene::block::Voxel;

struct Hit {
    t: f64,
    p: Vec3,
    n: Vec3,
    mat_id: usize,
}

pub struct Renderer {
    w: usize,
    h: usize,
    spp: usize,
    scene: Option<Scene>,
    camera: Option<CameraPose>,
    daynight: DayNight,
    use_procedural_sky: bool,
}

impl Renderer {
    pub fn new(w: usize, h: usize, spp: usize) -> Self {
        Self {
            w,
            h,
            spp: spp.max(1),
            scene: None,
            camera: None,
            daynight: DayNight::new(),
            use_procedural_sky: true,
        }
    }

    pub fn set_use_procedural_sky(&mut self, v: bool) {
        self.use_procedural_sky = v;
    }

    pub fn set_scene(&mut self, scene: &Scene) {
        self.scene = Some(scene.clone());
    }

    pub fn set_camera(&mut self, pose: &CameraPose) {
        self.camera = Some(*pose);
    }

    pub fn render_frame(&mut self, fb: &mut Framebuffer, time: f64) {
        let scene = match &self.scene {
            Some(s) => s,
            None => return,
        };
        let cam = match &self.camera {
            Some(c) => c,
            None => return,
        };

        let eye = cam.eye;
        let forward = (cam.target - cam.eye).normalized();
        let right = forward.cross(cam.up).normalized();
        let up = right.cross(forward);

        let fov_rad = cam.fov_deg.to_radians();
        let scale = (fov_rad * 0.5).tan();
        let aspect = self.w as f64 / self.h as f64;

        let inv_spp = 1.0 / (self.spp as f64);

        for y in 0..self.h {
            for x in 0..self.w {
                let mut col = Color::new(0.0, 0.0, 0.0);

                for s in 0..self.spp {
                    let fx = (x as f64 + 0.5 + s as f64 * inv_spp) / self.w as f64;
                    let fy = (y as f64 + 0.5 + s as f64 * inv_spp) / self.h as f64;

                    let px = (2.0 * fx - 1.0) * aspect * scale;
                    let py = (1.0 - 2.0 * fy) * scale;

                    let dir = (forward + right * px + up * py).normalized();
                    let ray = Ray::new(eye, dir);

                    col = col + self.trace_ray(&ray, scene, time);
                }

                col = col * inv_spp;
                fb.set(x, y, tone_map(col));
            }
        }
    }

    fn trace_ray(&self, ray: &Ray, scene: &Scene, time: f64) -> Color {
        if let Some(hit) = intersect_voxels(ray, &scene.voxels) {
            let mat = &scene.materials[hit.mat_id];

            let mut albedo = mat.albedo;
            if let Some(tex) = mat.texture.as_ref() {
                let (u, v) = compute_uv(&hit, mat.uv_scale);
                let tex_color = sample_texture(tex, u, v);
                albedo = Vec3::new(
                    albedo.x * tex_color.x,
                    albedo.y * tex_color.y,
                    albedo.z * tex_color.z,
                );
            }

            let sun_dir = self.daynight.sun_direction(time);
            let sun_col = self.daynight.sun_color(time);
            let sun_int = self.daynight.sun_intensity(time);
            let light_col = sun_col * sun_int;

            let ndotl = hit.n.dot(sun_dir).max(0.0);
            let diffuse = Vec3::new(
                albedo.x * ndotl * light_col.x,
                albedo.y * ndotl * light_col.y,
                albedo.z * ndotl * light_col.z,
            );

            let view_dir = (ray.o - hit.p).normalized();
            let half_vec = (sun_dir + view_dir).normalized();
            let n_dot_h = hit.n.dot(half_vec).max(0.0);
            let shininess = 32.0;
            let spec_strength = 0.5;
            let spec = n_dot_h.powf(shininess) * spec_strength;
            let specular = light_col * spec;

            let ambient = albedo * self.daynight.ambient_level(time);

            diffuse + specular + ambient
        } else {
            self.background(ray.d, time)
        }
    }

    fn background(&self, dir: Vec3, time: f64) -> Color {
        if !self.use_procedural_sky {
            return Vec3::new(0.5, 0.7, 1.0);
        }

        let dir_n = dir.normalized();

        let sky = self.daynight.sky_color(time);

        let sun_dir = self.daynight.sun_direction(time);
        let sun_col = self.daynight.sun_color(time);
        let sun_int = self.daynight.sun_intensity(time);

        let dot = clamp01(dir_n.dot(sun_dir));
        let sun_spot = dot.powf(600.0);
        let sun_glow = dot.powf(4.0);

        let base = sky * (1.0 - 0.3 * sun_glow);

        base + sun_col * sun_int * (4.0 * sun_spot + 0.3 * sun_glow)
    }
}

fn intersect_aabb(ray: &Ray, min: Vec3, max: Vec3) -> Option<(f64, Vec3)> {
    let inv_dir = Vec3::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);

    let mut tmin = (min.x - ray.o.x) * inv_dir.x;
    let mut tmax = (max.x - ray.o.x) * inv_dir.x;
    let mut n = Vec3::new(1.0, 0.0, 0.0);

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
        n = Vec3::new(-1.0, 0.0, 0.0);
    }

    let mut tymin = (min.y - ray.o.y) * inv_dir.y;
    let mut tymax = (max.y - ray.o.y) * inv_dir.y;
    let mut ny = Vec3::new(0.0, 1.0, 0.0);

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
        ny = Vec3::new(0.0, -1.0, 0.0);
    }

    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }

    if tymin > tmin {
        tmin = tymin;
        n = ny;
    }
    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (min.z - ray.o.z) * inv_dir.z;
    let mut tzmax = (max.z - ray.o.z) * inv_dir.z;
    let mut nz = Vec3::new(0.0, 0.0, 1.0);

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
        nz = Vec3::new(0.0, 0.0, -1.0);
    }

    if (tmin > tzmax) || (tzmin > tmax) {
        return None;
    }

    if tzmin > tmin {
        tmin = tzmin;
        n = nz;
    }
    if tzmax < tmax {
        tmax = tzmax;
    }

    if tmax < ray.tmin || tmin > ray.tmax {
        return None;
    }

    let t = if tmin < ray.tmin { tmax } else { tmin };
    if t < ray.tmin || t > ray.tmax {
        return None;
    }

    Some((t, n))
}

fn intersect_voxels(ray: &Ray, voxels: &[Voxel]) -> Option<Hit> {
    let mut best_t = ray.tmax;
    let mut best_hit: Option<Hit> = None;

    for v in voxels {
        if let Some((t, n)) = intersect_aabb(ray, v.min, v.max) {
            if t < best_t {
                best_t = t;
                let p = ray.at(t);
                best_hit = Some(Hit {
                    t,
                    p,
                    n,
                    mat_id: v.mat_id,
                });
            }
        }
    }

    best_hit
}

fn tone_map(c: Color) -> Color {
    let r = clamp01(c.x);
    let g = clamp01(c.y);
    let b = clamp01(c.z);
    Color::new(r, g, b)
}

fn compute_uv(hit: &Hit, uv_scale: f64) -> (f64, f64) {
    let p = hit.p;
    let n = hit.n;
    let (u, v) = if n.x.abs() > 0.5 {
        (p.z, p.y)
    } else if n.z.abs() > 0.5 {
        (p.x, p.y)
    } else {
        (p.x, p.z)
    };
    let u = u * uv_scale;
    let v = v * uv_scale;
    let u = u - u.floor();
    let v = v - v.floor();
    (u, v)
}

fn sample_texture(tex: &crate::scene::TextureCPU, u: f64, v: f64) -> Color {
    let x = (u * tex.width as f64) as usize % tex.width;
    let y = (v * tex.height as f64) as usize % tex.height;
    tex.data[y * tex.width + x]
}
