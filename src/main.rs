use std::fs;
use std::path::Path;

use crate::app::camera::{CameraPose, CameraOrbit};
use crate::core::image::Image;
use crate::core::vec3::Vec3;
use crate::render::renderer::Renderer;
use crate::scene::Scene;

mod app;
mod core;
mod render;
mod scene;

fn main() {
    // ====== MODO RÁPIDO PERO CON MÁS FRAMES ======
    let w = 640usize;       // antes: 960
    let h = 360usize;       // antes: 540
    let frames = 100usize;  // << aumentado (antes: 5 / 120)
    let fps = 30usize;      // un poco más fluido
    let spp = 1usize;       // mantener 1 para velocidad
    let outdir = "docs/demo/frames";

    println!("Config (FAST+LONG):");
    println!("  res:      {}x{}", w, h);
    println!("  frames:   {}", frames);
    println!("  fps:      {}", fps);
    println!("  spp:      {}", spp);
    println!("  outdir:   {}", outdir);

    fs::create_dir_all(outdir).ok();

    // === Cámara (órbita alrededor del centro de la casa) ===
    let center = Vec3::new(5.0, 3.0, 5.0);
    let mut orbit = CameraOrbit::new(center);
    orbit.base_radius = 18.0;
    orbit.zoom_amp = 0.0;
    orbit.height = 7.0;

    // === Escena: CASA VOXEL + (OBJ opcional) ===
    let mut scene: Scene = crate::scene::build_minecraft_house_scene();

    // OBJ opcional: solo si existe el archivo (no bloquea en modo rápido)
    let obj_path = "assets/models/bunny.obj";
    if Path::new(obj_path).exists() {
        println!("OBJ encontrado, se carga: {}", obj_path);
        let stone_mat_id = 2usize;
        let tris = crate::scene::mesh::load_obj_triangles(
            obj_path,
            stone_mat_id,
            0.5,
            Vec3::new(10.0, 1.0, 10.0),
        );
        scene.triangles.extend(tris);
    } else {
        println!("OBJ no encontrado (se omite): {}", obj_path);
    }

    // === Renderer ===
    let mut renderer = Renderer::new(w, h, spp);
    renderer.set_scene(&scene);

    // === Render loop ===
    let mut img = Image::new(w, h);

    for f in 0..frames {
        // Tiempo base (segundos) para cámara
        let t = (f as f64) / (fps as f64);

        // Órbita algo más rápida para que recorra bien en 240 frames
        let orbit_time = t * 1.5;
        let pose: CameraPose = orbit.pose_at(orbit_time);
        renderer.set_camera(&pose);

        // Ciclo día/noche progresivo: recorre ~2 días en toda la secuencia
        // (day_time en horas solares: 0..24)
        let day_progress = (f as f64) / (frames as f64); // 0..1
        let day_time = (day_progress * 48.0) % 24.0;     // 0..24 (dos ciclos)

        renderer.render_frame(&mut img, day_time);

        // Guarda cada frame
        let path = format!("{}/frame_{:04}.bmp", outdir, f);
        img.save_bmp(&path);
        println!("Saved {}", path);
    }

    println!("\nListo (FAST+LONG). Para video preview:");
    println!(
        "ffmpeg -framerate {} -i {}/frame_%04d.bmp -pix_fmt yuv420p docs/demo/diorama_long.mp4",
        fps, outdir
    );
}
