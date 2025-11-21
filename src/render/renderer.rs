use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::camera::CameraPose;
use crate::app::daynight::DayNight;
use crate::core::image::Image;
use crate::core::ray::Ray;
use crate::core::vec3::{Color, Vec3};
use crate::scene::Scene;
use crate::scene::voxel::Voxel;

use image; // para cargar JPG/PNG/BMP

/* ========================= util ========================= */

#[inline]
fn clamp01(v: Color) -> Color {
    Color::new(
        v.x.clamp(0.0, 1.0),
        v.y.clamp(0.0, 1.0),
        v.z.clamp(0.0, 1.0),
    )
}

#[inline]
fn tonemap_aces(c: Color) -> Color {
    let a = 2.51;
    let b = 0.03;
    let c1 = 2.43;
    let d = 0.59;
    let e = 0.14;
    Color::new(
        ((c.x * (a * c.x + b)) / (c.x * (c1 * c.x + d) + e))
            .max(0.0)
            .min(1.0),
        ((c.y * (a * c.y + b)) / (c.y * (c1 * c.y + d) + e))
            .max(0.0)
            .min(1.0),
        ((c.z * (a * c.z + b)) / (c.z * (c1 * c.z + d) + e))
            .max(0.0)
            .min(1.0),
    )
}

#[inline]
fn gamma22(c: Color) -> Color {
    let g = 1.0 / 2.2;
    Color::new(c.x.powf(g), c.y.powf(g), c.z.powf(g))
}

#[inline]
fn hadamard(a: Color, b: Color) -> Color {
    Color::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

/* ====================== Sol / muestreo ====================== */

fn sun_sample_dir(sun_dir: Vec3, i: u32) -> Vec3 {
    let n = sun_dir.normalized();
    let up = if n.y.abs() < 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let t = up.cross(n).normalized();
    let b = n.cross(t);

    let pts = [
        (0.0, 0.0),
        (0.6, 0.3),
        (0.0, -0.6),
        (-0.6, 0.3),
        (0.6, -0.3),
        (0.0, 0.6),
        (-0.6, -0.3),
    ];
    let (ux, uy) = pts[(i as usize) % pts.len()];
    let spread = 0.008;
    (n + t * (ux * spread) + b * (uy * spread)).normalized()
}

/* ====================== AO simplificado ====================== */

fn occlusion_ray_hit(ray: &Ray, voxels: &[Voxel], max_t: f64) -> bool {
    for v in voxels {
        if let Some((t0, _t1)) = ray_box_intersect(ray, v.min, v.max, max_t) {
            if t0 > ray.tmin && t0 < max_t {
                return true;
            }
        }
    }
    false
}

fn unoccluded_ray(ray: &Ray, voxels: &[Voxel], max_t: f64) -> bool {
    !occlusion_ray_hit(ray, voxels, max_t)
}

fn blocked_along(ray: &Ray, voxels: &[Voxel], tmax: f64) -> bool {
    let mut shadow = *ray;
    shadow.tmax = tmax;
    for v in voxels {
        if let Some((t0, _t1)) = ray_box_intersect(&shadow, v.min, v.max, tmax) {
            if t0 > shadow.tmin && t0 < shadow.tmax {
                return true;
            }
        }
    }
    false
}

fn bent_normal(p: Vec3, n: Vec3, voxels: &[Voxel]) -> Vec3 {
    let eps = 1e-3;
    let samples = [
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 1.0),
        Vec3::new(0.0, 1.0, -1.0),
    ];

    let mut b = Vec3::new(0.0, 0.0, 0.0);
    let mut cnt = 0.0;

    for s in samples.iter() {
        let dir = (*s).normalized();
        let r = Ray::new(p + n * eps, dir);
        if unoccluded_ray(&r, voxels, 1.0e6) {
            b = b + dir;
            cnt += 1.0;
        }
    }

    if cnt > 0.0 {
        (b * (1.0 / cnt)).normalized()
    } else {
        n
    }
}

fn ao_term(p: Vec3, n: Vec3, voxels: &[Voxel]) -> f64 {
    let mut occ: f64 = 0.0;
    let eps: f64 = 1e-3;

    let dirs = [
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.5, 1.0, 0.0),
        Vec3::new(-0.5, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.5),
        Vec3::new(0.0, 1.0, -0.5),
    ];

    for d in dirs.iter() {
        let dir = (*d).normalized();
        let r = Ray::new(p + n * eps, dir);
        if occlusion_ray_hit(&r, voxels, 1.0) {
            occ += 1.0;
        }
    }

    let occ_norm = occ / (dirs.len() as f64);
    (1.0 - 0.35 * occ_norm).clamp(0.4, 1.0)
}

/* ====================== Intersección AABB ====================== */

fn safe_inv(x: f64) -> f64 {
    if x.abs() < 1e-8 {
        1.0e8
    } else {
        1.0 / x
    }
}

fn ray_box_intersect(ray: &Ray, min: Vec3, max: Vec3, max_t: f64) -> Option<(f64, f64)> {
    let inv_dx = safe_inv(ray.d.x);
    let inv_dy = safe_inv(ray.d.y);
    let inv_dz = safe_inv(ray.d.z);

    let mut tmin = (min.x - ray.o.x) * inv_dx;
    let mut tmax = (max.x - ray.o.x) * inv_dx;
    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    let mut tymin = (min.y - ray.o.y) * inv_dy;
    let mut tymax = (max.y - ray.o.y) * inv_dy;
    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if tmin > tymax || tymin > tmax {
        return None;
    }

    if tymin > tmin {
        tmin = tymin;
    }
    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (min.z - ray.o.z) * inv_dz;
    let mut tzmax = (max.z - ray.o.z) * inv_dz;
    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
    }

    if tmin > tzmax || tzmin > tmax {
        return None;
    }

    if tzmin > tmin {
        tmin = tzmin;
    }
    if tzmax < tmax {
        tmax = tzmax;
    }

    if tmin < ray.tmin || tmin > max_t {
        None
    } else {
        Some((tmin, tmax))
    }
}

/* ====================== Renderer ====================== */

#[derive(Clone)]
struct Light {
    pos: Vec3,
    color: Color,
    intensity: f64,
}

#[derive(Clone)]
struct Tex {
    w: usize,
    h: usize,
    data: Vec<u8>, // RGB
}

pub struct Renderer {
    w: usize,
    h: usize,
    spp: usize,
    tilesz: usize,
    scene: Option<Scene>,
    camera: Option<CameraPose>,
    dn: DayNight,
    tex_cache: Vec<Option<Tex>>,
    skybox_cache: [Option<Tex>; 6],
    lights: Vec<Light>,
    use_procedural_sky: bool,
}

impl Renderer {
    pub fn new(w: usize, h: usize, spp: usize) -> Self {
        Self {
            w,
            h,
            spp,
            tilesz: 32,
            scene: None,
            camera: None,
            dn: DayNight::new(),
            tex_cache: Vec::new(),
            skybox_cache: [None, None, None, None, None, None],
            lights: Vec::new(),
            use_procedural_sky: true,
        }
    }

    pub fn set_use_procedural_sky(&mut self, v: bool) {
        self.use_procedural_sky = v;
    }

    pub fn set_scene(&mut self, scene: &Scene) {
        let cloned = scene.clone();

        let mut cache = Vec::with_capacity(cloned.materials.len());
        println!("\n== Texturas de materiales ==");
        for (i, m) in cloned.materials.iter().enumerate() {
            if let Some(path) = m.texture_path {
                let exists = Path::new(path).exists();
                println!(
                    "  [{}] {} -> {}  ({})",
                    i,
                    m.name,
                    path,
                    if exists { "existe" } else { "NO existe" }
                );
                let tex = load_tex(path);
                if let Some(ref t) = tex {
                    println!("       cargada OK ({}x{} RGB)", t.w, t.h);
                } else {
                    println!("       ERROR: no se pudo cargar imagen");
                }
                cache.push(tex);
            } else {
                println!("  [{}] {} -> (sin textura, solo albedo)", i, m.name);
                cache.push(None);
            }
        }
        self.tex_cache = cache;

        fn load_opt(path_opt: &Option<&'static str>) -> Option<Tex> {
            if let Some(p) = path_opt {
                let exists = Path::new(p).exists();
                println!(
                    "  skybox carga: {} ({})",
                    p,
                    if exists { "existe" } else { "NO existe" }
                );
                load_tex(p)
            } else {
                None
            }
        }
        let sb = &cloned.skybox;
        println!("\n== Skybox ==");
        self.skybox_cache = [
            load_opt(&sb.right),
            load_opt(&sb.left),
            load_opt(&sb.top),
            load_opt(&sb.bottom),
            load_opt(&sb.front),
            load_opt(&sb.back),
        ];

        let mut lights = Vec::new();
        for v in &cloned.voxels {
            let m = &cloned.materials[v.mat_id];
            if m.emissive.x > 0.0 || m.emissive.y > 0.0 || m.emissive.z > 0.0 {
                let center = (v.min + v.max) * 0.5;
                lights.push(Light {
                    pos: center,
                    color: Color::new(m.emissive.x, m.emissive.y, m.emissive.z),
                    intensity: 1.0,
                });
            }
        }
        self.lights = lights;

        self.scene = Some(cloned);
        println!("================================\n");
    }

    pub fn set_camera(&mut self, pose: &CameraPose) {
        self.camera = Some(CameraPose {
            eye: pose.eye,
            target: pose.target,
            up: pose.up,
            fov_deg: pose.fov_deg,
        });
    }

    pub fn render_frame(&mut self, img: &mut Image, time: f64) {
        let ntiles_x = (self.w + self.tilesz - 1) / self.tilesz;
        let ntiles_y = (self.h + self.tilesz - 1) / self.tilesz;

        let sun_dir = self.dn.sun_direction(time);
        let sun_intensity = self.dn.sun_intensity(time);
        let sun_color = self.dn.sun_color(time);
        let sky_color = self.dn.sky_color(time);
        let ambient_level = self.dn.ambient_level(time);

        let scene_cloned = self.scene.clone();
        let camera_cloned = self.camera.clone();
        let tex_cache_cloned = self.tex_cache.clone();
        let skybox_cache_cloned = self.skybox_cache.clone();
        let lights_cloned = self.lights.clone();
        let time_local = time;

        let fb = Arc::new(Mutex::new(vec![
            Color::new(0.0, 0.0, 0.0);
            self.w * self.h
        ]));

        let mut handles = Vec::new();

        for ty in 0..ntiles_y {
            for tx in 0..ntiles_x {
                let fb_cl = Arc::clone(&fb);
                let w = self.w;
                let h = self.h;
                let tilesz = self.tilesz;
                let spp = self.spp;

                let sun_dir_local = sun_dir;
                let sun_intensity_local = sun_intensity;
                let sun_color_local = sun_color;
                let sky_color_local = sky_color;
                let ambient_level_local = ambient_level;
                let use_procedural_sky_local = self.use_procedural_sky;

                let scene_local = scene_cloned.clone();
                let cam_local = camera_cloned.clone();
                let tex_cache_local = tex_cache_cloned.clone();
                let skybox_cache_local = skybox_cache_cloned.clone();
                let lights_local = lights_cloned.clone();

                let handle = thread::spawn(move || {
                    let x0 = tx * tilesz;
                    let y0 = ty * tilesz;
                    let x1 = (x0 + tilesz).min(w);
                    let y1 = (y0 + tilesz).min(h);

                    let mut tile_colors: Vec<(usize, usize, Color)> =
                        Vec::with_capacity((x1 - x0) * (y1 - y0));

                    if scene_local.is_none() || cam_local.is_none() {
                        for y in y0..y1 {
                            for x in x0..x1 {
                                let v = y as f64 / (h - 1).max(1) as f64;
                                let base = Color::new(
                                    sky_color_local.x * (1.0 - v * 0.3),
                                    sky_color_local.y * (1.0 - v * 0.3),
                                    sky_color_local.z,
                                );
                                tile_colors.push((x, y, base));
                            }
                        }
                    } else {
                        let scene = scene_local.unwrap();
                        let cam = cam_local.unwrap();
                        let pose = cam;

                        for y in y0..y1 {
                            for x in x0..x1 {
                                let mut color_acc = Color::new(0.0, 0.0, 0.0);

                                for _s in 0..spp {
                                    let ray = make_primary_ray(x, y, w, h, &pose);

                                    if let Some(hit) = trace_voxels(&ray, &scene.voxels) {
                                        let mat = &scene.materials[hit.mat_id];

                                        let (mut u, mut v) =
                                            voxel_uv(hit.vmin, hit.vmax, hit.p, hit.n);
                                        let uvscale = if mat.uv_scale.is_finite() {
                                            mat.uv_scale
                                        } else {
                                            1.0
                                        };
                                        u *= uvscale;
                                        v *= uvscale;
                                        if mat.animated_uv {
                                            u = (u + time_local * 0.2).fract();
                                            v = v.fract();
                                        }

                                        let mut albedo = clamp01(mat.albedo);
                                        if let Some(tex) =
                                            tex_for_mat(hit.mat_id, &tex_cache_local)
                                        {
                                            let tex_c = sample_tex_nearest(tex, u, v);
                                            albedo = clamp01(hadamard(albedo, tex_c));
                                        }

                                        let nrm = hit.n.normalized();

                                        // luz solar
                                        let mut sun_contribution =
                                            Color::new(0.0, 0.0, 0.0);
                                        if sun_intensity_local > 0.0 {
                                            let samples = 4;
                                            let mut sun_lit = 0.0;
                                            for i in 0..samples {
                                                let l =
                                                    sun_sample_dir(sun_dir_local, i as u32);
                                                let nl = nrm.dot(l).max(0.0);
                                                if nl > 0.0 {
                                                    let eps = 1e-4;
                                                    let vis =
                                                        if unoccluded_ray(
                                                            &Ray::new(
                                                                hit.p + nrm * eps,
                                                                l,
                                                            ),
                                                            &scene.voxels,
                                                            1e6,
                                                        ) {
                                                            1.0
                                                        } else {
                                                            0.0
                                                        };
                                                    sun_lit += nl * vis;
                                                }
                                            }
                                            sun_lit /= samples as f64;

                                            let sun_rgb = Color::new(
                                                sun_color_local.x,
                                                sun_color_local.y,
                                                sun_color_local.z,
                                            );
                                            sun_contribution = hadamard(
                                                albedo,
                                                sun_rgb,
                                            ) * (sun_lit * sun_intensity_local * 1.0);
                                        }

                                        // ambiente hemisférico
                                        let sky_up = Color::new(
                                            sky_color_local.x,
                                            sky_color_local.y,
                                            sky_color_local.z,
                                        );
                                        let ground_col =
                                            Color::new(0.08, 0.07, 0.06);
                                        let k_hemi =
                                            (nrm.y * 0.5 + 0.5).clamp(0.0, 1.0);
                                        let hemi = sky_up * k_hemi
                                            + ground_col * (1.0 - k_hemi);
                                        let ambient =
                                            hadamard(albedo, hemi) * ambient_level_local;

                                        // AO
                                        let ao = ao_term(hit.p, nrm, &scene.voxels);

                                        // especular solar
                                        let mut specular =
                                            Color::new(0.0, 0.0, 0.0);
                                        if sun_intensity_local > 0.3 {
                                            let view = (-ray.d).normalized();
                                            let mut sun_vec = sun_dir_local;
                                            if sun_vec.y < 0.1 {
                                                sun_vec.y = 0.1;
                                            }
                                            let half_vec =
                                                (view + sun_vec).normalized();
                                            let nh = nrm.dot(half_vec).max(0.0);
                                            let shininess = 32.0;
                                            let spec_strength = 0.15;
                                            let spec_factor =
                                                nh.powf(shininess) * spec_strength;
                                            let sun_rgb = Color::new(
                                                sun_color_local.x,
                                                sun_color_local.y,
                                                sun_color_local.z,
                                            );
                                            specular =
                                                hadamard(sun_rgb, albedo) * spec_factor;
                                        }

                                        // luces emisivas
                                        let mut lights_sum =
                                            Color::new(0.0, 0.0, 0.0);
                                        for light in &lights_local {
                                            let to_l = light.pos - hit.p;
                                            let dist = to_l.length();
                                            let ldir = to_l / dist;

                                            let nl = nrm.dot(ldir).max(0.0);
                                            if nl <= 0.0 {
                                                continue;
                                            }

                                            let eps = 1e-4;
                                            let unoccluded = !blocked_along(
                                                &Ray::new(hit.p + nrm * eps, ldir),
                                                &scene.voxels,
                                                dist - eps,
                                            );
                                            if !unoccluded {
                                                continue;
                                            }

                                            let max_range = 10.0;
                                            let falloff =
                                                (1.0 - (dist / max_range).min(1.0))
                                                    .max(0.0);
                                            let atten = falloff * falloff;

                                            // flicker usando time_local
                                            let phase = time_local * 6.0
                                                + light.pos.x * 2.0
                                                + light.pos.z * 3.0;
                                            let flicker = (0.8
                                                + 0.2
                                                    * (phase.sin()
                                                        * (phase * 1.3).cos()))
                                                .clamp(0.6, 1.2);

                                            let contrib = hadamard(
                                                albedo,
                                                light.color
                                                    * (light.intensity * flicker),
                                            ) * (nl * atten * 0.8);
                                            lights_sum = lights_sum + contrib;
                                        }

                                        let mut c = (ambient + sun_contribution
                                            + lights_sum
                                            + specular)
                                            * ao;

                                        let min_light = ambient_level_local * 0.3;
                                        c = c + (albedo * min_light);

                                        color_acc = color_acc + c;
                                    } else {
                                        // miss: cielo
                                        if use_procedural_sky_local {
                                            let up = ray.d.y.clamp(-1.0, 1.0);
                                            let base = Color::new(
                                                sky_color_local.x,
                                                sky_color_local.y,
                                                sky_color_local.z,
                                            );

                                            let t_h = ((up + 1.0) * 0.5)
                                                .clamp(0.0, 1.0);
                                            let horizon = Color::new(
                                                base.x * 1.05,
                                                base.y * 1.05,
                                                base.z * 1.05,
                                            );
                                            let zenith = Color::new(
                                                base.x * 0.85,
                                                base.y * 0.90,
                                                base.z * 1.0,
                                            );
                                            let mut sky = zenith * t_h
                                                + horizon * (1.0 - t_h);

                                            let dp =
                                                ray.d.dot(sun_dir_local).clamp(-1.0, 1.0);
                                            let ang = dp.acos();
                                            let sun_disk =
                                                (0.008 - ang).max(0.0) * 80.0;
                                            let sun_glow =
                                                (0.10 - ang).max(0.0) * 1.5;
                                            let sun_rgb = Color::new(
                                                sun_color_local.x,
                                                sun_color_local.y,
                                                sun_color_local.z,
                                            );
                                            sky = sky + sun_rgb
                                                * (sun_disk + sun_glow)
                                                * sun_intensity_local;

                                            color_acc = color_acc + sky;
                                        } else {
                                            let (face, su, sv) =
                                                dir_to_cube_uv(ray.d);
                                            if let Some(tex) =
                                                &skybox_cache_local[face]
                                            {
                                                let c =
                                                    sample_tex_nearest(tex, su, sv);
                                                color_acc = color_acc + c;
                                            } else {
                                                let v = y as f64
                                                    / (h - 1).max(1) as f64;
                                                let base = Color::new(
                                                    sky_color_local.x
                                                        * (1.0 - v * 0.3),
                                                    sky_color_local.y
                                                        * (1.0 - v * 0.3),
                                                    sky_color_local.z,
                                                );
                                                color_acc = color_acc + base;
                                            }
                                        }
                                    }
                                }

                                let c = color_acc / (spp as f64);
                                tile_colors.push((x, y, c));
                            }
                        }
                    }

                    if let Ok(mut fb_guard) = fb_cl.lock() {
                        for (x, y, c) in tile_colors {
                            let idx = y * w + x;
                            fb_guard[idx] = c;
                        }
                    }
                });
                handles.push(handle);
            }
        }

        for h in handles {
            let _ = h.join();
        }

        // Tomar el framebuffer y pasarlo al Image
        let fb_data = fb.lock().unwrap();
        for y in 0..self.h {
            for x in 0..self.w {
                let idx = y * self.w + x;
                let mut out = fb_data[idx];
                out = tonemap_aces(out);
                out = gamma22(out);
                img.set(x, y, out);
            }
        }
    }
}

/* ====================== Helpers de Ray Tracing ====================== */

#[derive(Clone, Copy)]
struct HitInfo {
    t: f64,
    p: Vec3,
    n: Vec3,
    mat_id: usize,
    vmin: Vec3,
    vmax: Vec3,
}

fn make_primary_ray(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    cam: &CameraPose,
) -> Ray {
    let aspect = w as f64 / h as f64;
    let fov = cam.fov_deg.to_radians();
    let scale = (fov * 0.5).tan();

    let px = (2.0 * ((x as f64 + 0.5) / w as f64) - 1.0) * aspect * scale;
    let py = (1.0 - 2.0 * ((y as f64 + 0.5) / h as f64)) * scale;

    let forward = (cam.target - cam.eye).normalized();
    let right = forward.cross(cam.up).normalized();
    let up = right.cross(forward).normalized();

    let dir = (forward + right * px + up * py).normalized();

    let mut ray = Ray::new(cam.eye, dir);
    ray.tmin = 0.001;
    ray.tmax = 1e6;
    ray
}

fn trace_voxels(ray: &Ray, voxels: &[Voxel]) -> Option<HitInfo> {
    let mut closest_t = ray.tmax;
    let mut best: Option<HitInfo> = None;

    for v in voxels {
        if let Some((t0, _t1)) = ray_box_intersect(ray, v.min, v.max, closest_t) {
            if t0 < closest_t && t0 > ray.tmin {
                closest_t = t0;
                let p = ray.o + ray.d * t0;
                let n = voxel_normal_at(p, v.min, v.max);
                best = Some(HitInfo {
                    t: t0,
                    p,
                    n,
                    mat_id: v.mat_id,
                    vmin: v.min,
                    vmax: v.max,
                });
            }
        }
    }
    best
}

fn voxel_normal_at(p: Vec3, min: Vec3, max: Vec3) -> Vec3 {
    let dxmin = (p.x - min.x).abs();
    let dxmax = (p.x - max.x).abs();
    let dymin = (p.y - min.y).abs();
    let dymax = (p.y - max.y).abs();
    let dzmin = (p.z - min.z).abs();
    let dzmax = (p.z - max.z).abs();

    let mut best = dxmin;
    let mut n = Vec3::new(-1.0, 0.0, 0.0);

    if dxmax < best {
        best = dxmax;
        n = Vec3::new(1.0, 0.0, 0.0);
    }
    if dymin < best {
        best = dymin;
        n = Vec3::new(0.0, -1.0, 0.0);
    }
    if dymax < best {
        best = dymax;
        n = Vec3::new(0.0, 1.0, 0.0);
    }
    if dzmin < best {
        best = dzmin;
        n = Vec3::new(0.0, 0.0, -1.0);
    }
    if dzmax < best {
        best = dzmax;
        n = Vec3::new(0.0, 0.0, 1.0);
    }
    n
}

/* ====================== Skybox mapping ====================== */

fn dir_to_cube_uv(d: Vec3) -> (usize, f64, f64) {
    let ax = d.x.abs();
    let ay = d.y.abs();
    let az = d.z.abs();

    let (face, sc, tc, ma) = if ax >= ay && ax >= az {
        if d.x > 0.0 {
            (0usize, -d.z, -d.y, ax)
        } else {
            (1usize, d.z, -d.y, ax)
        }
    } else if ay >= ax && ay >= az {
        if d.y > 0.0 {
            (2usize, d.x, d.z, ay)
        } else {
            (3usize, d.x, -d.z, ay)
        }
    } else if d.z > 0.0 {
        (4usize, d.x, -d.y, az)
    } else {
        (5usize, -d.x, -d.y, az)
    };

    let u = 0.5 * (sc / ma + 1.0);
    let v = 0.5 * (tc / ma + 1.0);
    (face, u, v)
}

/* ====================== Texturas ====================== */

fn load_tex(path: &str) -> Option<Tex> {
    let img = image::open(path).ok()?.to_rgb8();
    let (w, h) = img.dimensions();
    let data = img.into_raw();

    Some(Tex {
        w: w as usize,
        h: h as usize,
        data,
    })
}

fn sample_tex_nearest(tex: &Tex, mut u: f64, mut v: f64) -> Color {
    u = u.fract();
    if u < 0.0 {
        u += 1.0;
    }
    v = v.fract();
    if v < 0.0 {
        v += 1.0;
    }

    let x = (u * tex.w as f64)
        .floor()
        .clamp(0.0, (tex.w - 1) as f64) as usize;
    let y = (v * tex.h as f64)
        .floor()
        .clamp(0.0, (tex.h - 1) as f64) as usize;
    let idx = (y * tex.w + x) * 3;

    let r = tex.data[idx] as f64 / 255.0;
    let g = tex.data[idx + 1] as f64 / 255.0;
    let b = tex.data[idx + 2] as f64 / 255.0;
    Color::new(r, g, b)
}

fn tex_for_mat<'a>(mat_id: usize, cache: &'a [Option<Tex>]) -> Option<&'a Tex> {
    if mat_id < cache.len() {
        cache[mat_id].as_ref()
    } else {
        None
    }
}

/* ========== UV helper (ajusta si ya lo tienes en otro lado) ========== */

fn voxel_uv(_min: Vec3, _max: Vec3, p: Vec3, n: Vec3) -> (f64, f64) {
    let (u, v) = if n.x.abs() > n.y.abs() && n.x.abs() > n.z.abs() {
        (p.z, p.y)
    } else if n.y.abs() > n.z.abs() {
        (p.x, p.z)
    } else {
        (p.x, p.y)
    };
    (u as f64, v as f64)
}
