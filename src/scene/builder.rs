use crate::core::vec3::Vec3;
use crate::scene::{Material, Portal, Scene, Skybox};
use crate::scene::voxel::Voxel;
use crate::scene::mesh; // opcional: para cargar .obj si quieres

/// helper para insertar una caja AABB como vóxel
fn add_box(scene: &mut Scene, min: Vec3, max: Vec3, mat_id: usize) {
    scene.voxels.push(Voxel { min, max, mat_id });
}

/// Escena estilo “Minecraft”: terreno + casa con paredes delgadas,
/// techo escalonado, puerta y ventanas, agua animada, antorchas emisivas,
/// skybox y (opcional) un OBJ de ejemplo.
pub fn build_minecraft_house_scene() -> Scene {
    let mut scene = Scene::new();

    /* ===================== 1) Materiales ===================== */
    // Apuntan a assets/textures/*.bmp (los que generamos)
    let grass  = Material::new("grass",  Vec3::new(0.95, 0.98, 0.95), Some("assets/textures/grass.bmp"))
        .with_uv_scale(8.0)
        .with_specular(0.03);

    let dirt   = Material::new("dirt",   Vec3::new(0.55, 0.44, 0.36), Some("assets/textures/dirt.bmp"))
        .with_uv_scale(4.0)
        .with_specular(0.02);

    let stone  = Material::new("stone",  Vec3::new(0.72, 0.72, 0.74), Some("assets/textures/stone.bmp"))
        .with_uv_scale(3.0)
        .with_specular(0.06);

    let planks = Material::new("planks", Vec3::new(0.85, 0.70, 0.52), Some("assets/textures/planks.bmp"))
        .with_uv_scale(2.5)
        .with_specular(0.05);

    // franja oscura
    let dark_wood = Material::new("dark_wood", Vec3::new(0.35, 0.25, 0.18), Some("assets/textures/planks.bmp"))
        .with_uv_scale(2.5)
        .with_specular(0.04);

    let roof   = Material::new("roof",   Vec3::new(0.95, 0.60, 0.55), Some("assets/textures/roof.bmp"))
        .with_uv_scale(2.0)
        .with_specular(0.04);

    let glass  = Material::new("glass",  Vec3::new(0.95, 0.97, 1.0),   Some("assets/textures/glass.bmp"))
        .with_uv_scale(1.0)
        .with_specular(0.20)
        .with_reflection(0.05); // leve reflejo si luego lo soporta el renderer

    let water  = Material::new("water",  Vec3::new(0.25, 0.45, 0.95), Some("assets/textures/water.bmp"))
        .with_uv_scale(6.0)
        .animated(true)
        .with_specular(0.12);

    let torch  = Material::new("torch",  Vec3::new(1.00, 0.85, 0.45), None)
        .with_emissive(Vec3::new(4.0, 2.6, 1.2));

    scene.materials.extend([
        grass,      // 0
        dirt,       // 1
        stone,      // 2
        planks,     // 3
        dark_wood,  // 4
        roof,       // 5
        glass,      // 6
        water,      // 7
        torch,      // 8
    ]);

    /* ===================== 2) Skybox (opcional) ===================== */
    scene.skybox = Skybox {
        right:  Some("assets/skybox/right.bmp"),   // +X
        left:   Some("assets/skybox/left.bmp"),    // -X
        top:    Some("assets/skybox/top.bmp"),     // +Y
        bottom: Some("assets/skybox/bottom.bmp"),  // -Y
        front:  Some("assets/skybox/front.bmp"),   // +Z
        back:   Some("assets/skybox/back.bmp"),    // -Z
    };

    /* ===================== 3) Terreno (dirt + grass) ===================== */
    add_box(&mut scene, Vec3::new(-5.0, 0.0, -5.0), Vec3::new(20.0, 0.8, 20.0), 1); // dirt
    add_box(&mut scene, Vec3::new(-5.0, 0.8, -5.0), Vec3::new(20.0, 1.0, 20.0), 0); // grass

    /* ===================== 4) Casa (paredes delgadas) ===================== */
    // Dimensiones generales
    let x0 = 3.0;  let x1 = 13.0;
    let z0 = 3.0;  let z1 = 13.0;
    let y0 = 1.0;  let y1 = 6.0;
    let t  = 0.25; // grosor pared

    // Pared trasera (Z = z0)
    add_box(&mut scene, Vec3::new(x0, y0, z0),         Vec3::new(x1, y1, z0 + t), 3);
    // Pared delantera con hueco de puerta (Z = z1)
    let door_x0 = 7.4; let door_x1 = 8.6; let door_h = 2.2;
    add_box(&mut scene, Vec3::new(x0,      y0, z1 - t), Vec3::new(door_x0, y1, z1), 3);
    add_box(&mut scene, Vec3::new(door_x1, y0, z1 - t), Vec3::new(x1,      y1, z1), 3);
    // dintel
    add_box(&mut scene, Vec3::new(door_x0, y0 + door_h, z1 - t), Vec3::new(door_x1, y1, z1), 3);
    // Laterales
    add_box(&mut scene, Vec3::new(x0,     y0, z0), Vec3::new(x0 + t, y1, z1), 3);
    add_box(&mut scene, Vec3::new(x1 - t, y0, z0), Vec3::new(x1,     y1, z1), 3);

    // Franja oscura (resalta el volumen y corta la planicie de la pared)
    let band_h = 0.7;
    add_box(&mut scene, Vec3::new(x0, y0 + 2.2, z0),         Vec3::new(x1, y0 + 2.2 + band_h, z0 + t), 4);
    add_box(&mut scene, Vec3::new(x0, y0 + 2.2, z1 - t),     Vec3::new(x1, y0 + 2.2 + band_h, z1),     4);
    add_box(&mut scene, Vec3::new(x0, y0 + 2.2, z0),         Vec3::new(x0 + t, y0 + 2.2 + band_h, z1), 4);
    add_box(&mut scene, Vec3::new(x1 - t, y0 + 2.2, z0),     Vec3::new(x1,     y0 + 2.2 + band_h, z1), 4);

    /* ===================== 5) Ventanas (vidrio) ===================== */
    // Frontales
    add_box(&mut scene, Vec3::new(6.2, 2.0, z1 - t), Vec3::new(7.2, 3.5, z1), 6);
    add_box(&mut scene, Vec3::new(8.8, 2.0, z1 - t), Vec3::new(9.8, 3.5, z1), 6);
    // Laterales
    add_box(&mut scene, Vec3::new(x0,     2.0, 7.0), Vec3::new(x0 + t, 3.5, 9.0), 6);
    add_box(&mut scene, Vec3::new(x1 - t, 2.0, 7.0), Vec3::new(x1,     3.5, 9.0), 6);

    /* ===================== 6) Techo escalonado (roof) ===================== */
    // Elevado un poco para evitar z-fighting con paredes superiores
    let y_top = y1 + 0.06;
    add_box(&mut scene, Vec3::new(2.5,  y_top,        2.5), Vec3::new(13.5, y_top + 0.6, 13.5), 5);
    add_box(&mut scene, Vec3::new(3.5,  y_top + 0.6,  3.5), Vec3::new(12.5, y_top + 1.2, 12.5), 5);
    add_box(&mut scene, Vec3::new(4.5,  y_top + 1.2,  4.5), Vec3::new(11.5, y_top + 1.8, 11.5), 5);
    add_box(&mut scene, Vec3::new(5.5,  y_top + 1.8,  5.5), Vec3::new(10.5, y_top + 2.6, 10.5), 5);

    /* ===================== 7) Antorchas (emisivas) ===================== */
    add_box(&mut scene, Vec3::new(6.0, 3.2, z1 - 0.15), Vec3::new(6.2, 3.6, z1), 8);
    add_box(&mut scene, Vec3::new(9.8, 3.2, z1 - 0.15), Vec3::new(10.0,3.6, z1), 8);

    /* ===================== 8) Agua animada ===================== */
    add_box(&mut scene, Vec3::new(1.0, 1.0, 14.0), Vec3::new(4.5, 1.2, 17.0), 7);

    /* ===================== 9) Portales (opcional) ===================== */
    scene.portals.push(Portal {
        min: Vec3::new(3.0, 1.0, 12.0),
        max: Vec3::new(3.2, 3.6, 12.6),
        to_pos: Vec3::new(12.8, 2.0, 3.4),
        rot_y_deg: 180.0,
    });
    scene.portals.push(Portal {
        min: Vec3::new(12.8, 1.0, 3.0),
        max: Vec3::new(13.0, 3.6, 3.6),
        to_pos: Vec3::new(3.1, 2.0, 12.3),
        rot_y_deg: 180.0,
    });

    /* ===================== 10) OBJ opcional ===================== */
    let tris = mesh::load_obj_triangles(
        "assets/models/bunny.obj",
        2,                           // stone
        0.6,
        Vec3::new(15.0, 1.0, 10.0),  // posición
    );
    scene.triangles.extend(tris);

    scene
}
