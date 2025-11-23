use raylib::prelude::*;

use crate::app::camera::CameraController;
use crate::core::math::{Vec3, to_u8};
use crate::framebuffer::Framebuffer;
use crate::render::renderer::Renderer;
use crate::scene::builder::build_minecraft_house_scene;

mod app;
mod core;
mod render;
mod scene;
mod framebuffer;

fn main() {
    let width: usize = 960;
    let height: usize = 540;
    let spp: usize = 4;

    let (mut rl, thread) = raylib::init()
        .size(width as i32, height as i32)
        .title("Proyecto 2 - Raytracer (Isa)")
        .build();

    rl.set_target_fps(30);

    let mut renderer = Renderer::new(width, height, spp);
    renderer.set_use_procedural_sky(true);

    let scene = build_minecraft_house_scene();
    renderer.set_scene(&scene);

    let orbit_center = Vec3::new(8.0, 3.0, 8.0);
    let mut camera = CameraController::new(orbit_center);

    let mut fb = Framebuffer::new(width, height);

    let mut t: f64 = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time() as f64;
        t += dt * 15.0;
        let day_time = t;

        let left = rl.is_key_down(KeyboardKey::KEY_LEFT);
        let right = rl.is_key_down(KeyboardKey::KEY_RIGHT);
        let zoom_in = rl.is_key_down(KeyboardKey::KEY_UP);
        let zoom_out = rl.is_key_down(KeyboardKey::KEY_DOWN);
        let height_up = rl.is_key_down(KeyboardKey::KEY_PAGE_UP);
        let height_down = rl.is_key_down(KeyboardKey::KEY_PAGE_DOWN);

        camera.apply_input(
            left,
            right,
            zoom_in,
            zoom_out,
            height_up,
            height_down,
            dt,
        );
        let cam_pose = camera.pose();
        renderer.set_camera(&cam_pose);

        renderer.render_frame(&mut fb, day_time);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for y in 0..height {
            for x in 0..width {
                let c = fb.data[y * width + x];
                let r = to_u8(c.x);
                let g = to_u8(c.y);
                let b = to_u8(c.z);
                d.draw_pixel(x as i32, y as i32, Color::new(r, g, b, 255));
            }
        }
    }
}
