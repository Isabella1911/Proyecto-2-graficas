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

/* ========================= util ========================= */

#[inline]
fn hadamard(a: Color, b: Color) -> Color {
    Color::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

#[inline]
fn saturate(x: f64) -> f64 { x.max(0.0).min(1.0) }

#[inline]
fn clamp01(v: Color) -> Color {
    Color::new(v.x.clamp(0.0, 1.0), v.y.clamp(0.0, 1.0), v.z.clamp(0.0, 1.0))
}

#[inline]
fn tonemap_aces(c: Color) -> Color {
    let a = 2.51;
    let b = 0.03;
    let c1 = 2.43;
    let d = 0.59;
    let e = 0.14;
    Color::new(
        ((c.x * (a * c.x + b)) / (c.x * (c1 * c.x + d) + e)).max(0.0).min(1.0),
        ((c.y * (a * c.y + b)) / (c.y * (c1 * c.y + d) + e)).max(0.0).min(1.0),
        ((c.z * (a * c.z + b)) / (c.z * (c1 * c.z + d) + e)).max(0.0).min(1.0),
    )
}

#[inline]
fn gamma22(c: Color) -> Color {
    let g = 1.0 / 2.2;
    Color::new(c.x.powf(g), c.y.powf(g), c.z.powf(g))
}

/// Sombras suaves del sol (determinístico)
fn sun_sample_dir(sun_dir: Vec3, i: u32) -> Vec3 {
    let n = sun_dir.normalized();
    let up = if n.y.abs() < 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let t = n.cross(up).normalized();
    let b = t.cross(n).normalized();

    let pts = [
        (0.0f64, 0.0f64),
        (0.6f64, 0.0f64),
        (0.0f64, 0.6f64),
        (-0.6f64, -0.3f64),
    ];
    let (ux, uy) = pts[(i as usize) % pts.len()];
    let spread = 0.008f64; // Sombras más definidas estilo Minecraft
    (n + t * (ux * spread) + b * (uy * spread)).normalized()
}

/// AO simplificado estilo Minecraft (más sutil)
fn ao_term(p: Vec3, n: Vec3, voxels: &[Voxel]) -> f64 {
    let mut occ: f64 = 0.0;
    let eps: f64 = 1e-3;
    let step: f64 = 0.15;

    for k in 1..=3 {
        let dist: f64 = step * (k as f64);
        let o = p + n * (eps + dist);
        let r = Ray::new(o, n);
        if blocked_along(&r, voxels, 0.25) { 
            occ += 1.0 / (k as f64); // AO decae con distancia
        }
    }

    (1.0 - 0.22 * occ).clamp(0.5, 1.0)
}

/* ========================= Renderer ========================= */

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
    data: Vec<u8>,
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

    // ====== NUEVO: activar cielo procedural (ignora skybox si true) ======
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
            use_procedural_sky: true, // por defecto ON
        }
    }

    pub fn set_scene(&mut self, scene: &Scene) {
        let cloned = scene.clone();

        let mut cache = Vec::with_capacity(cloned.materials.len());
        println!("\n== Texturas de materiales ==");
        for (i, m) in cloned.materials.iter().enumerate() {
            if let Some(path) = m.texture_path {
                let exists = Path::new(path).exists();
                println!("  [{}] {} -> {}  ({})",
                    i, m.name, path, if exists { "existe" } else { "NO existe" });
                let tex = load_bmp24(path);
                if let Some(ref t) = tex {
                    println!("       cargada OK ({}x{} 24bpp)", t.w, t.h);
                } else {
                    println!("       ERROR: no se pudo cargar BMP");
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
                println!("  skybox carga: {} ({})", p, if exists { "existe" } else { "NO existe" });
                load_bmp24(p)
            } else { None }
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
                    color: m.emissive, 
                    intensity: 2.0 // Más intensas estilo antorchas Minecraft
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

    /// Permite alternar entre cielo procedural y skybox
    pub fn set_use_procedural_sky(&mut self, v: bool) {
        self.use_procedural_sky = v;
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

        let fb = Arc::new(Mutex::new(vec![Color::new(0.0, 0.0, 0.0); self.w * self.h]));

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
                        // Fondo con sky color
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
                        let pose = cam_local.unwrap();

                        for y in y0..y1 {
                            for x in x0..x1 {
                                let mut color_acc = Color::new(0.0, 0.0, 0.0);

                                for _s in 0..spp {
                                    let ray = make_primary_ray(x, y, w, h, &pose);

                                    if let Some(hit) = trace_voxels(&ray, &scene.voxels) {
                                        let mat = &scene.materials[hit.mat_id];

                                        // UV con tiling/anim
                                        let (mut u, mut v) = voxel_uv(hit.vmin, hit.vmax, hit.p, hit.n);
                                        let uvscale = if mat.uv_scale.is_finite() { mat.uv_scale } else { 1.0 };
                                        u *= uvscale;
                                        v *= uvscale;
                                        if mat.animated_uv {
                                            u = (u + time * 0.2).fract();
                                            v = v.fract();
                                        }

                                        // Albedo base (clamp) y textura opcional
                                        let mut albedo = clamp01(mat.albedo);
                                        if let Some(tex) = tex_for_mat(hit.mat_id, &tex_cache_local) {
                                            let tex_c = sample_tex_nearest(tex, u, v);
                                            albedo = clamp01(hadamard(albedo, tex_c));
                                        }

                                        let nrm = hit.n.normalized();

                                        // === LUZ SOLAR (balanceada) ===
                                        let mut sun_contribution = Color::new(0.0, 0.0, 0.0);
                                        if sun_intensity_local > 0.01 {
                                            let mut sun_vec = sun_dir_local;
                                            if sun_vec.y < 0.1 { sun_vec.y = 0.1; }
                                            sun_vec = sun_vec.normalized();

                                            // Sombras suaves
                                            let samples: u32 = 3;
                                            let mut sun_lit: f64 = 0.0;
                                            for i in 0..samples {
                                                let l = sun_sample_dir(sun_vec, i);
                                                let nl = nrm.dot(l).max(0.0);
                                                if nl > 0.0 {
                                                    let eps = 1e-4;
                                                    let vis = if !blocked_along(&Ray::new(hit.p + nrm * eps, l), &scene.voxels, 1e6) { 1.0 } else { 0.0 };
                                                    sun_lit += nl * vis;
                                                }
                                            }
                                            sun_lit /= samples as f64;

                                            let sun_rgb = Color::new(sun_color_local.x, sun_color_local.y, sun_color_local.z);
                                            // ↓↓↓ Bajado de 1.5 a 1.0 para evitar quemado
                                            sun_contribution = hadamard(albedo, sun_rgb) * (sun_lit * sun_intensity_local * 1.0);
                                        }

                                        // === AMBIENTE HEMISFÉRICO ===
                                        let sky_up = Color::new(sky_color_local.x, sky_color_local.y, sky_color_local.z);
                                        let ground_col = Color::new(0.08, 0.07, 0.06);
                                        let k_hemi = (nrm.y * 0.5 + 0.5).clamp(0.0, 1.0);
                                        let hemi = sky_up * k_hemi + ground_col * (1.0 - k_hemi);
                                        let ambient = hadamard(albedo, hemi) * ambient_level_local;

                                        // === AO ===
                                        let ao = ao_term(hit.p, nrm, &scene.voxels);

                                        // === ESPECULAR SUTIL ===
                                        let mut specular = Color::new(0.0, 0.0, 0.0);
                                        if sun_intensity_local > 0.3 {
                                            let view = (-ray.d).normalized();
                                            let mut sun_vec = sun_dir_local;
                                            if sun_vec.y < 0.1 { sun_vec.y = 0.1; }
                                            sun_vec = sun_vec.normalized();
                                            let hdir = (view + sun_vec).normalized();
                                            let spec = nrm.dot(hdir).max(0.0).powf(32.0);
                                            let ks: f64 = 0.06;
                                            let sun_rgb = Color::new(sun_color_local.x, sun_color_local.y, sun_color_local.z);
                                            specular = sun_rgb * (ks * spec * sun_intensity_local);
                                        }

                                        // === ANTORCHAS (point lights) ===
                                        let mut lights_sum = Color::new(0.0, 0.0, 0.0);
                                        for light in &lights_local {
                                            let to_l = light.pos - hit.p;
                                            let dist = to_l.length();
                                            if dist <= 1e-6 { continue; }
                                            let ldir = to_l / dist;

                                            let nl = nrm.dot(ldir).max(0.0);
                                            if nl <= 0.0 { continue; }

                                            let eps = 1e-4;
                                            let unoccluded = !blocked_along(&Ray::new(hit.p + nrm * eps, ldir), &scene.voxels, dist - eps);
                                            if !unoccluded { continue; }

                                            // Atenuación estilo Minecraft (alcance ~10-12 bloques)
                                            let max_range = 10.0;
                                            let falloff = (1.0 - (dist / max_range).min(1.0)).max(0.0);
                                            let atten = falloff * falloff; // Cuadrático
                                            
                                            let contrib = hadamard(albedo, light.color * light.intensity) * (nl * atten * 0.8);
                                            lights_sum = lights_sum + contrib;
                                        }

                                        // === COMBINACIÓN FINAL ===
                                        let mut c = sun_contribution + ambient * ao + specular + lights_sum + clamp01(mat.emissive);
                                        
                                        // Luz mínima (nunca completamente negro)
                                        let min_light = ambient_level_local * 0.3;
                                        c = c + (albedo * min_light);

                                        color_acc = color_acc + c;
                                    } else {
                                        // Miss: cielo procedural o skybox
                                        if use_procedural_sky_local {
                                            // Gradiente según dirección y hora
                                            let up = ray.d.y.clamp(-1.0, 1.0);
                                            let base = Color::new(sky_color_local.x, sky_color_local.y, sky_color_local.z);

                                            // Mezcla zenit/horizonte
                                            let t_h = ((up + 1.0) * 0.5).clamp(0.0, 1.0); // 0 = abajo, 1 = arriba
                                            let horizon = Color::new(base.x * 1.05, base.y * 1.05, base.z * 1.05);
                                            let zenith  = Color::new(base.x * 0.85, base.y * 0.90, base.z * 1.05);
                                            let mut sky = zenith * t_h + horizon * (1.0 - t_h);

                                            // Sol: disco + glow
                                            let dp = ray.d.dot(sun_dir_local).clamp(-1.0, 1.0);
                                            let ang = dp.acos();
                                            let sun_disk = (0.008 - ang).max(0.0) * 80.0; // pequeño y brillante
                                            let sun_glow = (0.10  - ang).max(0.0) * 1.5;  // halo ancho
                                            let sun_rgb = Color::new(sun_color_local.x, sun_color_local.y, sun_color_local.z);
                                            sky = sky + sun_rgb * (sun_disk + sun_glow) * sun_intensity_local;

                                            color_acc = color_acc + sky;
                                        } else {
                                            let (face, su, sv) = dir_to_cube_uv(ray.d);
                                            if let Some(tex) = &skybox_cache_local[face] {
                                                let c = sample_tex_nearest(tex, su, sv);
                                                color_acc = color_acc + c;
                                            } else {
                                                let v = y as f64 / (h - 1).max(1) as f64;
                                                let base = Color::new(
                                                    sky_color_local.x * (1.0 - v * 0.3),
                                                    sky_color_local.y * (1.0 - v * 0.3),
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
            h.join().unwrap();
        }

        let fb_data: Vec<Color> = match Arc::try_unwrap(fb) {
            Ok(mutex) => mutex.into_inner().unwrap(),
            Err(arc) => arc.lock().unwrap().clone(),
        };

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

fn make_primary_ray(x: usize, y: usize, w: usize, h: usize, pose: &CameraPose) -> Ray {
    let fov = pose.fov_deg.to_radians();
    let aspect = w as f64 / h as f64;

    let px = ((x as f64 + 0.5) / w as f64) * 2.0 - 1.0;
    let py = 1.0 - ((y as f64 + 0.5) / h as f64) * 2.0;

    let tan = (fov * 0.5).tan();

    let forward = (pose.target - pose.eye).normalized();
    let right = forward.cross(pose.up).normalized();
    let up = right.cross(forward).normalized();

    let dir = (forward + right * (px * aspect * tan) + up * (py * tan)).normalized();
    Ray::new(pose.eye, dir)
}

fn trace_voxels(ray: &Ray, voxels: &[Voxel]) -> Option<HitInfo> {
    let mut closest_t = ray.tmax;
    let mut best: Option<HitInfo> = None;

    for v in voxels {
        if let Some((t0, _t1)) = intersect_aabb(ray, v.min, v.max) {
            if t0 < closest_t && t0 > ray.tmin {
                let p = ray.at(t0);
                let n = voxel_normal_at(p, v.min, v.max);
                closest_t = t0;
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

    let mut n = Vec3::new(0.0, 0.0, 0.0);
    let mut d = f64::INFINITY;

    if dxmin < d { d = dxmin; n = Vec3::new(-1.0, 0.0, 0.0); }
    if dxmax < d { d = dxmax; n = Vec3::new( 1.0, 0.0, 0.0); }
    if dymin < d { d = dymin; n = Vec3::new(0.0, -1.0, 0.0); }
    if dymax < d { d = dymax; n = Vec3::new(0.0,  1.0, 0.0); }
    if dzmin < d { d = dzmin; n = Vec3::new(0.0, 0.0, -1.0); }
    if dzmax < d {              n = Vec3::new(0.0, 0.0,  1.0); }
    n
}

fn voxel_uv(min: Vec3, _max: Vec3, p: Vec3, n: Vec3) -> (f64, f64) {
    let ax = n.x.abs();
    let ay = n.y.abs();
    let az = n.z.abs();

    if ax >= ay && ax >= az {
        let u = p.z - min.z;
        let v = p.y - min.y;
        return (u, v);
    }
    if ay >= ax && ay >= az {
        let u = p.x - min.x;
        let v = p.z - min.z;
        return (u, v);
    }
    let u = p.x - min.x;
    let v = p.y - min.y;
    (u, v)
}

/* ============================== Intersección ============================== */

fn intersect_aabb(ray: &Ray, min: Vec3, max: Vec3) -> Option<(f64, f64)> {
    let mut tmin = ray.tmin;
    let mut tmax = ray.tmax;

    for i in 0..3 {
        let (o, d, minv, maxv) = match i {
            0 => (ray.o.x, ray.d.x, min.x, max.x),
            1 => (ray.o.y, ray.d.y, min.y, max.y),
            _ => (ray.o.z, ray.d.z, min.z, max.z),
        };
        if d.abs() < 1e-12 {
            if o < minv || o > maxv { return None; }
        } else {
            let mut t1 = (minv - o) / d;
            let mut t2 = (maxv - o) / d;
            if t1 > t2 { std::mem::swap(&mut t1, &mut t2); }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax { return None; }
        }
    }
    Some((tmin, tmax))
}

#[inline]
fn blocked_along(ray: &Ray, voxels: &[Voxel], tmax: f64) -> bool {
    let mut shadow = *ray;
    shadow.tmax = tmax;
    for v in voxels {
        if let Some((t0, _)) = intersect_aabb(&shadow, v.min, v.max) {
            if t0 > shadow.tmin && t0 < shadow.tmax { return true; }
        }
    }
    false
}

/* ================================ Skybox ================================ */

fn dir_to_cube_uv(d: Vec3) -> (usize, f64, f64) {
    let ax = d.x.abs();
    let ay = d.y.abs();
    let az = d.z.abs();

    let (face, sc, tc, ma) = if ax >= ay && ax >= az {
        if d.x > 0.0 { (0usize, -d.z, -d.y, ax) } else { (1usize,  d.z, -d.y, ax) }
    } else if ay >= ax && ay >= az {
        if d.y > 0.0 { (2usize,  d.x,  d.z, ay) } else { (3usize,  d.x, -d.z, ay) }
    } else {
        if d.z > 0.0 { (4usize,  d.x, -d.y, az) } else { (5usize, -d.x, -d.y, az) }
    };

    let u = 0.5 * (sc / ma + 1.0);
    let v = 0.5 * (tc / ma + 1.0);
    (face, u, v)
}

/* ================================ Texturas ================================ */

fn load_bmp24(path: &str) -> Option<Tex> {
    let mut f = File::open(path).ok()?;
    let mut header = [0u8; 14];
    f.read_exact(&mut header).ok()?;
    if &header[0..2] != b"BM" { return None; }
    let pixel_offset = u32::from_le_bytes([header[10], header[11], header[12], header[13]]) as u64;

    let mut dib = [0u8; 40];
    f.read_exact(&mut dib).ok()?;
    let width  = i32::from_le_bytes([dib[4], dib[5], dib[6], dib[7]]);
    let height = i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]);
    let bpp    = u16::from_le_bytes([dib[14], dib[15]]);
    if bpp != 24 || width == 0 || height == 0 { return None; }

    let w = width.unsigned_abs() as usize;
    let h = height.unsigned_abs() as usize;

    f.seek(SeekFrom::Start(pixel_offset)).ok()?;
    let row_stride = ((w * 3 + 3) / 4) * 4;
    let mut raw = vec![0u8; row_stride * h];
    f.read_exact(&mut raw).ok()?;

    let mut data = vec![0u8; w * h * 3];
    if height > 0 {
        for y in 0..h {
            let src = (h - 1 - y) * row_stride;
            let dst = y * w * 3;
            data[dst..dst + w * 3].copy_from_slice(&raw[src..src + w * 3]);
        }
    } else {
        for y in 0..h {
            let src = y * row_stride;
            let dst = y * w * 3;
            data[dst..dst + w * 3].copy_from_slice(&raw[src..src + w * 3]);
        }
    }

    Some(Tex { w, h, data })
}

fn sample_tex_nearest(tex: &Tex, mut u: f64, mut v: f64) -> Color {
    u = u.fract(); if u < 0.0 { u += 1.0; }
    v = v.fract(); if v < 0.0 { v += 1.0; }

    let x = (u * tex.w as f64).floor().clamp(0.0, (tex.w - 1) as f64) as usize;
    let y = (v * tex.h as f64).floor().clamp(0.0, (tex.h - 1) as f64) as usize;
    let idx = (y * tex.w + x) * 3;

    let b = tex.data[idx] as f64 / 255.0;
    let g = tex.data[idx + 1] as f64 / 255.0;
    let r = tex.data[idx + 2] as f64 / 255.0;
    Color::new(r, g, b)
}

fn tex_for_mat<'a>(mat_id: usize, cache: &'a [Option<Tex>]) -> Option<&'a Tex> {
    if mat_id < cache.len() { cache[mat_id].as_ref() } else { None }
}
