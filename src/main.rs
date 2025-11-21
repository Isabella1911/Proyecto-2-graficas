use std::fs;
use std::path::Path;

use crate::app::camera::CameraOrbit;
use crate::core::image::Image;
use crate::core::vec3::Vec3;
use crate::render::renderer::Renderer;
use crate::scene::builder::build_minecraft_house_scene;

mod app;
mod core;
mod render;
mod scene;

fn main() {
    // Resolución y samples
    let width: usize = 960;
    let height: usize = 540;
    let spp: usize = 16;

    // Config de animación
    let fps: f64 = 30.0;
    let seconds: f64 = 10.0;          // duración del timelapse
    let nframes: u32 = (fps * seconds) as u32;

    // Carpeta de salida
    let outdir = "docs/demo/frames_long";
    if !Path::new(outdir).exists() {
        fs::create_dir_all(outdir).expect("no se pudo crear carpeta de salida");
    }

    // Renderer
    let mut renderer = Renderer::new(width, height, spp);
    renderer.set_use_procedural_sky(true); // usar DayNight (cielo procedural)

    // Escena
    let scene = build_minecraft_house_scene();
    renderer.set_scene(&scene);

    // ====== CÁMARA ORBITAL ======
    // Orbitando alrededor del centro de la casa (~8,3,8)
    let orbit = CameraOrbit::new(Vec3::new(8.0, 3.0, 8.0));

    let mut img = Image::new(width, height);

    for f in 0..nframes {
        // Tiempo en segundos desde el inicio
        let t = f as f64 / fps;

        
        let day_time = t * 12.0; 

        // Cámara para este instante (usa t normal para que la órbita vaya suave)
        let cam_pose = orbit.pose_at(t);
        renderer.set_camera(&cam_pose);

        // Render
        renderer.render_frame(&mut img, day_time);

        // Guardar frame
        let path = format!("{}/frame_{:04}.bmp", outdir, f);
        img.save_bmp(&path);
        println!("Saved {}", path);
    }

    println!("\nListo. Generados {} frames en {}", nframes, outdir);
}
