use crate::core::vec3::Vec3;
use crate::scene::{Material, Portal, Scene, Skybox};
use crate::scene::voxel::Voxel;
use crate::scene::mesh;

fn add_box(scene: &mut Scene, min: Vec3, max: Vec3, mat_id: usize) {
    scene.voxels.push(Voxel { min, max, mat_id });
}

pub fn build_minecraft_house_scene() -> Scene {
    let mut scene = Scene::new();

    let grass = Material::new("grass", Vec3::new(0.95, 0.98, 0.95), Some("assets/textures/grass.jpeg"))
        .with_uv_scale(8.0)
        .with_specular(0.03);

    let dirt = Material::new("dirt", Vec3::new(0.55, 0.44, 0.36), Some("assets/textures/dirt.jpeg"))
        .with_uv_scale(4.0)
        .with_specular(0.02);

    let stone = Material::new("stone", Vec3::new(0.72, 0.72, 0.74), Some("assets/textures/stone.jpeg"))
        .with_uv_scale(3.0)
        .with_specular(0.06);

    let planks = Material::new("planks", Vec3::new(0.85, 0.70, 0.52), Some("assets/textures/planks.jpeg"))
        .with_uv_scale(2.5)
        .with_specular(0.05);

    let dark_wood = Material::new("dark_wood", Vec3::new(0.35, 0.25, 0.18), Some("assets/textures/planks.jpeg"))
        .with_uv_scale(2.5)
        .with_specular(0.04);

    let roof = Material::new("roof", Vec3::new(0.95, 0.60, 0.55), Some("assets/textures/roof.jpeg"))
        .with_uv_scale(2.0)
        .with_specular(0.04);

    let glass = Material::new("glass", Vec3::new(0.95, 0.97, 1.0), Some("assets/textures/glass.jpeg"))
        .with_uv_scale(1.0)
        .with_specular(0.6)
        .with_reflection(0.25);

    let water = Material::new("water", Vec3::new(0.25, 0.45, 0.95), Some("assets/textures/water.png"))
        .with_uv_scale(6.0)
        .animated(true)
        .with_specular(0.12);

    let torch = Material::new("torch", Vec3::new(1.00, 0.85, 0.45), None)
        .with_emissive(Vec3::new(4.0, 2.6, 1.2));

    let tree_leaves = Material::new("tree_leaves", Vec3::new(0.65, 0.85, 0.60), Some("assets/textures/tree.jpeg"))
        .with_uv_scale(2.0)
        .with_specular(0.02);

    let sun = Material::new("sun", Vec3::new(1.0, 0.95, 0.85), None)
        .with_emissive(Vec3::new(20.0, 18.0, 10.0));

    scene.materials.extend([
        grass,
        dirt,
        stone,
        planks,
        dark_wood,
        roof,
        glass,
        water,
        torch,
        tree_leaves,
        sun,
    ]);

    scene.skybox = Skybox {
        right: None,
        left: None,
        top: None,
        bottom: None,
        front: None,
        back: None,
    };

    add_box(&mut scene, Vec3::new(-5.0, 0.0, -5.0), Vec3::new(20.0, 0.8, 20.0), 1);
    add_box(&mut scene, Vec3::new(-5.0, 0.8, -5.0), Vec3::new(20.0, 1.0, 20.0), 0);

    let x0 = 3.0;
    let x1 = 13.0;
    let z0 = 3.0;
    let z1 = 13.0;
    let y0 = 1.0;
    let y1 = 6.0;
    let t = 0.25;

    add_box(&mut scene, Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z0 + t), 3);

    add_box(
        &mut scene,
        Vec3::new(6.5, 2.0, z0),
        Vec3::new(9.5, 4.0, z0 + t),
        6,
    );

    let door_x0 = 7.4;
    let door_x1 = 8.6;
    let door_h = 2.2;
    add_box(&mut scene, Vec3::new(x0, y0, z1 - t), Vec3::new(door_x0, y1, z1), 3);
    add_box(&mut scene, Vec3::new(door_x1, y0, z1 - t), Vec3::new(x1, y1, z1), 3);

    add_box(
        &mut scene,
        Vec3::new(door_x0, y0 + door_h, z1 - t),
        Vec3::new(door_x1, y1, z1),
        3,
    );

    add_box(&mut scene, Vec3::new(x0, y0, z0), Vec3::new(x0 + t, y1, z1), 3);
    add_box(&mut scene, Vec3::new(x1 - t, y0, z0), Vec3::new(x1, y1, z1), 3);

    let band_h = 0.7;
    add_box(
        &mut scene,
        Vec3::new(x0, y0 + 2.2, z0),
        Vec3::new(x1, y0 + 2.2 + band_h, z0 + t),
        4,
    );
    add_box(
        &mut scene,
        Vec3::new(x0, y0 + 2.2, z1 - t),
        Vec3::new(x1, y0 + 2.2 + band_h, z1),
        4,
    );
    add_box(
        &mut scene,
        Vec3::new(x0, y0 + 2.2, z0),
        Vec3::new(x0 + t, y0 + 2.2 + band_h, z1),
        4,
    );
    add_box(
        &mut scene,
        Vec3::new(x1 - t, y0 + 2.2, z0),
        Vec3::new(x1, y0 + 2.2 + band_h, z1),
        4,
    );

    add_box(&mut scene, Vec3::new(6.2, 2.0, z1 - t), Vec3::new(7.2, 3.5, z1), 6);
    add_box(&mut scene, Vec3::new(8.8, 2.0, z1 - t), Vec3::new(9.8, 3.5, z1), 6);

    add_box(
        &mut scene,
        Vec3::new(9.0, 3.8, z1 - t),
        Vec3::new(9.8, 5.0, z1),
        6,
    );

    add_box(
        &mut scene,
        Vec3::new(x0, 2.0, 7.0),
        Vec3::new(x0 + t, 3.5, 9.0),
        6,
    );
    add_box(
        &mut scene,
        Vec3::new(x1 - t, 2.0, 7.0),
        Vec3::new(x1, 3.5, 9.0),
        6,
    );

    let y_top = y1 + 0.06;
    add_box(
        &mut scene,
        Vec3::new(2.5, y_top, 2.5),
        Vec3::new(13.5, y_top + 0.6, 13.5),
        5,
    );
    add_box(
        &mut scene,
        Vec3::new(3.5, y_top + 0.6, 3.5),
        Vec3::new(12.5, y_top + 1.2, 12.5),
        5,
    );
    add_box(
        &mut scene,
        Vec3::new(4.5, y_top + 1.2, 4.5),
        Vec3::new(11.5, y_top + 1.8, 11.5),
        5,
    );
    add_box(
        &mut scene,
        Vec3::new(5.5, y_top + 1.8, 5.5),
        Vec3::new(10.5, y_top + 2.6, 10.5),
        5,
    );

    add_box(
        &mut scene,
        Vec3::new(door_x0 + 0.05, y0, z1 - t + 0.02),
        Vec3::new(door_x1 - 0.05, y0 + door_h, z1 - 0.02),
        4,
    );

    add_box(
        &mut scene,
        Vec3::new(6.0, 3.2, z1 - 0.15),
        Vec3::new(6.2, 3.6, z1),
        8,
    );
    add_box(
        &mut scene,
        Vec3::new(9.8, 3.2, z1 - 0.15),
        Vec3::new(10.0, 3.6, z1),
        8,
    );

    add_box(
        &mut scene,
        Vec3::new(7.0, 1.0, z1 + 0.6),
        Vec3::new(7.3, 2.4, z1 + 0.9),
        4,
    );
    add_box(
        &mut scene,
        Vec3::new(6.9, 2.4, z1 + 0.5),
        Vec3::new(7.4, 2.9, z1 + 1.0),
        8,
    );

    add_box(
        &mut scene,
        Vec3::new(1.0, 1.0, 14.0),
        Vec3::new(4.5, 1.2, 17.0),
        7,
    );

    add_box(
        &mut scene,
        Vec3::new(15.8, 1.0, 8.2),
        Vec3::new(16.2, 5.5, 8.6),
        4,
    );

    add_box(
        &mut scene,
        Vec3::new(14.8, 5.5, 7.2),
        Vec3::new(17.2, 7.5, 9.6),
        9,
    );

    add_box(
        &mut scene,
        Vec3::new(15.3, 7.5, 7.7),
        Vec3::new(16.7, 8.9, 9.1),
        9,
    );

    add_box(
        &mut scene,
        Vec3::new(15.6, 8.9, 8.0),
        Vec3::new(16.4, 9.7, 8.8),
        9,
    );

    add_box(
        &mut scene,
        Vec3::new(30.0, 25.0, 5.0),
        Vec3::new(33.5, 28.5, 8.5),
        10,
    );

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

    let tris = mesh::load_obj_triangles(
        "assets/models/bunny.obj",
        2,
        0.6,
        Vec3::new(15.0, 1.0, 10.0),
    );
    scene.triangles.extend(tris);

    scene
}
