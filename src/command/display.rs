use freecrime::resources::parsers;
use freecrime::resources::types::map::Map;
use freecrime::resources::types::style::Style;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::core_pipeline::tonemapping::Tonemapping;
use std::fs;

pub fn execute(
    map_path: &str,
    style_path: &str,
    initial_pos_arr: Option<[f32; 3]>,
    initial_rot_arr: Option<[f32; 3]>,
) -> anyhow::Result<()> {
    let map_data = fs::read(map_path)?;
    let style_data = fs::read(style_path)?;

    let map = parsers::cmp::parse_cmp(&map_data)?;
    let style = parsers::gry::parse_gry(&style_data)?;

    let pos = initial_pos_arr.map(Vec3::from_array).unwrap_or(Vec3::new(128.0, 150.0, 128.0));
    let rot = initial_rot_arr.map(|a| Quat::from_euler(
        EulerRot::YXZ,
        a[0].to_radians(),
        a[1].to_radians(),
        a[2].to_radians(),
    )).unwrap_or(Quat::from_euler(EulerRot::YXZ, 0.0, -90.0f32.to_radians(), 0.0));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FreeCrime 3D Map Viewer".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(MapData { map, style })
        .insert_resource(AnimationTicks(0))
        .insert_resource(InitialCamera { pos, rot })
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement_system, animation_system))
        .run();

    Ok(())
}

#[derive(Resource)]
struct MapData {
    map: Map,
    style: Style,
}

#[derive(Resource)]
struct InitialCamera {
    pos: Vec3,
    rot: Quat,
}

#[derive(Resource)]
struct AnimationTicks(u64);

#[derive(Component)]
struct Chunk {
    cx: usize,
    cy: usize,
    has_animations: bool,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    map_data: Res<MapData>,
    initial_camera: Res<InitialCamera>,
) {
    // 1. Create Texture Atlas
    let tile_size = 64;
    let atlas_tiles_per_row = 32;
    let atlas_size = tile_size * atlas_tiles_per_row;
    let mut data = vec![0u8; atlas_size * atlas_size * 4];

    // Direct mapping: Map Index 0 maps to Atlas Index 0
    let mut current_atlas_idx = 0;

    use freecrime::resources::types::style::FaceType;

    // Sides
    for face_idx in 0..map_data.style.side_count {
        if current_atlas_idx >= 1024 { break; }
        let rgba = map_data.style.get_face_rgba(face_idx, FaceType::Side, 0);
        copy_to_atlas(&mut data, atlas_size, current_atlas_idx, &rgba);
        current_atlas_idx += 1;
    }

    // Lids (4 remaps each)
    for face_idx in 0..map_data.style.lid_count {
        for remap in 0..4 {
            if current_atlas_idx >= 1024 { break; }
            let rgba = map_data.style.get_face_rgba(face_idx, FaceType::Lid, remap);
            copy_to_atlas(&mut data, atlas_size, current_atlas_idx, &rgba);
            current_atlas_idx += 1;
        }
    }

    // Aux (4 remaps each)
    for face_idx in 0..map_data.style.aux_count {
        for remap in 0..4 {
            if current_atlas_idx >= 1024 { break; }
            let rgba = map_data.style.get_face_rgba(face_idx, FaceType::Aux, remap);
            copy_to_atlas(&mut data, atlas_size, current_atlas_idx, &rgba);
            current_atlas_idx += 1;
        }
    }

    let atlas_image = Image::new(
        Extent3d { width: atlas_size as u32, height: atlas_size as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    let atlas_handle = images.add(atlas_image);

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        ..default()
    });

    // 2. Generate Map Meshes
    for cy in 0..16 {
        for cx in 0..16 {
            if let Some((mesh, has_animations)) = generate_chunk_mesh(&map_data, cx, cy, atlas_tiles_per_row, 0) {
                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::from_xyz(cx as f32 * 16.0, 0.0, cy as f32 * 16.0),
                    Chunk { cx, cy, has_animations },
                ));
            }
        }
    }

    // 3. Camera
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Transform {
            translation: initial_camera.pos,
            rotation: initial_camera.rot,
            ..default()
        },
    ));

    println!("Display ready.");
    println!("Controls: WASD (move), Q/E (Roll), PageUp/Down (altitude)");
    println!("          Arrows Up/Down (Pitch), Arrows Left/Right (Yaw)");
    println!("          L: Log current position and angle");
}

fn copy_to_atlas(atlas: &mut [u8], atlas_size: usize, idx: usize, rgba: &[u8]) {
    let tiles_per_row = atlas_size / 64;
    let tx = (idx % tiles_per_row) * 64;
    let ty = (idx / tiles_per_row) * 64;
    for y in 0..64 {
        for x in 0..64 {
            let src_idx = (y * 64 + x) * 4;
            let dst_idx = ((ty + y) * atlas_size + (tx + x)) * 4;
            atlas[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
        }
    }
}

fn generate_chunk_mesh(map_data: &MapData, cx: usize, cy: usize, tiles_per_row: usize, ticks: u64) -> Option<(Mesh, bool)> {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut has_animations = false;

    for y in 0..16 {
        for x in 0..16 {
            let world_x = cx * 16 + x;
            let world_y = cy * 16 + y;
            let col = &map_data.map.grid[world_y * 256 + world_x];

            for z in 0..6 {
                let bt_idx = col.levels[z];
                if bt_idx == 0 { continue; }
                let bt = &map_data.map.block_types[bt_idx as usize];
                let fx = x as f32;
                let fy = -(z as f32);
                let fz = y as f32;
                let (d1, d2, d3, d4) = bt.get_slope_deltas();

                // Lid
                if bt.lid != 0 {
                    if map_data.style.is_block_animated(bt.lid as usize, 1) { has_animations = true; }
                    let remap = ((bt.type_map_ext >> 3) & 0x3) as usize;
                    let atlas_idx = map_data.style.get_animated_atlas_idx(bt.lid as usize, 1, remap, ticks);
                    add_face(&mut positions, &mut normals, &mut uvs, &mut indices,
                        [Vec3::new(fx, fy - d1, fz), Vec3::new(fx+1.0, fy - d2, fz), Vec3::new(fx+1.0, fy - d3, fz+1.0), Vec3::new(fx, fy - d4, fz+1.0)],
                        Vec3::Y, atlas_idx, tiles_per_row, bt.lid_rotation(), false, [0.0, 0.0, 1.0, 1.0]);
                }

                // Sides
                let is_flat = bt.is_flat();
                let flip_tb = (bt.type_map_ext & 0x20) == 0;
                let flip_lr = (bt.type_map_ext & 0x40) == 0;

                // Left Wall (West, NEG_X)
                if bt.left != 0 {
                    if map_data.style.is_block_animated(bt.left as usize, 0) { has_animations = true; }
                    let atlas_idx = map_data.style.get_animated_atlas_idx(bt.left as usize, 0, 0, ticks);
                    add_face(&mut positions, &mut normals, &mut uvs, &mut indices,
                        [Vec3::new(fx, fy - d4, fz+1.0), Vec3::new(fx, fy - d1, fz), Vec3::new(fx, fy-1.0, fz), Vec3::new(fx, fy-1.0, fz+1.0)],
                        Vec3::NEG_X, atlas_idx, tiles_per_row, 0, flip_lr, [d4, d1, 1.0, 1.0]);
                }
                // Right Wall (East, POS_X)
                if bt.right != 0 && !is_flat {
                    if map_data.style.is_block_animated(bt.right as usize, 0) { has_animations = true; }
                    let atlas_idx = map_data.style.get_animated_atlas_idx(bt.right as usize, 0, 0, ticks);
                    add_face(&mut positions, &mut normals, &mut uvs, &mut indices,
                        [Vec3::new(fx+1.0, fy - d2, fz), Vec3::new(fx+1.0, fy - d3, fz+1.0), Vec3::new(fx+1.0, fy-1.0, fz+1.0), Vec3::new(fx+1.0, fy-1.0, fz)],
                        Vec3::X, atlas_idx, tiles_per_row, 0, flip_lr, [d2, d3, 1.0, 1.0]);
                }
                // Top Wall (North, NEG_Z)
                if bt.top != 0 {
                    if map_data.style.is_block_animated(bt.top as usize, 0) { has_animations = true; }
                    let atlas_idx = map_data.style.get_animated_atlas_idx(bt.top as usize, 0, 0, ticks);
                    add_face(&mut positions, &mut normals, &mut uvs, &mut indices,
                        [Vec3::new(fx, fy - d1, fz), Vec3::new(fx+1.0, fy - d2, fz), Vec3::new(fx+1.0, fy-1.0, fz), Vec3::new(fx, fy-1.0, fz)],
                        Vec3::NEG_Z, atlas_idx, tiles_per_row, 0, flip_tb, [d1, d2, 1.0, 1.0]);
                }
                // Bottom Wall (South, POS_Z)
                if bt.bottom != 0 && !is_flat {
                    if map_data.style.is_block_animated(bt.bottom as usize, 0) { has_animations = true; }
                    let atlas_idx = map_data.style.get_animated_atlas_idx(bt.bottom as usize, 0, 0, ticks);
                    add_face(&mut positions, &mut normals, &mut uvs, &mut indices,
                        [Vec3::new(fx+1.0, fy - d3, fz+1.0), Vec3::new(fx, fy - d4, fz+1.0), Vec3::new(fx, fy-1.0, fz+1.0), Vec3::new(fx+1.0, fy-1.0, fz+1.0)],
                        Vec3::Z, atlas_idx, tiles_per_row, 0, flip_tb, [d3, d4, 1.0, 1.0]);
                }
            }
        }
    }

    if positions.is_empty() { return None; }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    Some((mesh, has_animations))
}

#[allow(clippy::too_many_arguments)]
fn add_face(pos: &mut Vec<Vec3>, norm: &mut Vec<Vec3>, uvs: &mut Vec<Vec2>, indices: &mut Vec<u32>, vertices: [Vec3; 4], n: Vec3, tile_idx: usize, tiles_per_row: usize, rot: u8, flip_h: bool, v_weights: [f32; 4]) {
    let start_idx = pos.len() as u32;
    pos.extend_from_slice(&vertices);
    for _ in 0..4 { norm.push(n); }

    let tiles_f = tiles_per_row as f32;
    let u_min = (tile_idx % tiles_per_row) as f32 / tiles_f;
    let v_min = (tile_idx / tiles_per_row) as f32 / tiles_f;
    let u_max = u_min + 1.0 / tiles_f;
    let v_max = v_min + 1.0 / tiles_f;

    let (mut u0, mut u1) = (u_min, u_max);
    if flip_h { std::mem::swap(&mut u0, &mut u1); }

    let mut face_uvs = [
        Vec2::new(u0, v_min + v_weights[0] * (v_max - v_min)),
        Vec2::new(u1, v_min + v_weights[1] * (v_max - v_min)),
        Vec2::new(u1, v_min + v_weights[2] * (v_max - v_min)),
        Vec2::new(u0, v_min + v_weights[3] * (v_max - v_min)),
    ];
    face_uvs.rotate_right(rot as usize % 4);
    uvs.extend_from_slice(&face_uvs);

    indices.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2, start_idx, start_idx + 2, start_idx + 3]);
}

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let Ok(mut transform) = query.get_single_mut() else { return; };

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let pos = transform.translation;
        println!("Current Camera:");
        println!("  --camera-position {:.2},{:.2},{:.2}", pos.x, pos.y, pos.z);
        println!("  --camera-rotation {:.2},{:.2},{:.2}", yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees());
    }

    let mut direction = Vec3::ZERO;
    let speed = 80.0;

    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let h_forward = -Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let h_right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

    if keyboard_input.pressed(KeyCode::KeyW) { direction += h_forward; }
    if keyboard_input.pressed(KeyCode::KeyS) { direction -= h_forward; }
    if keyboard_input.pressed(KeyCode::KeyA) { direction -= h_right; }
    if keyboard_input.pressed(KeyCode::KeyD) { direction += h_right; }
    if keyboard_input.pressed(KeyCode::PageUp) { direction += Vec3::Y; }
    if keyboard_input.pressed(KeyCode::PageDown) { direction -= Vec3::Y; }

    transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();

    let rot_speed = 1.5;
    // Arrows Left/Right -> Yaw (Global Y)
    if keyboard_input.pressed(KeyCode::ArrowLeft) { transform.rotate_y(rot_speed * time.delta_secs()); }
    if keyboard_input.pressed(KeyCode::ArrowRight) { transform.rotate_y(-rot_speed * time.delta_secs()); }

    // Arrows Up/Down -> Pitch (Local X)
    if keyboard_input.pressed(KeyCode::ArrowUp) { transform.rotate_local_x(rot_speed * time.delta_secs()); }
    if keyboard_input.pressed(KeyCode::ArrowDown) { transform.rotate_local_x(-rot_speed * time.delta_secs()); }

    // Q/E -> Roll (Local Z)
    if keyboard_input.pressed(KeyCode::KeyQ) { transform.rotate_local_z(rot_speed * time.delta_secs()); }
    if keyboard_input.pressed(KeyCode::KeyE) { transform.rotate_local_z(-rot_speed * time.delta_secs()); }
}

fn animation_system(
    time: Res<Time>,
    mut ticks: ResMut<AnimationTicks>,
    query: Query<(&Chunk, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
    map_data: Res<MapData>,
) {
    let new_ticks = (time.elapsed_secs() * 20.0).round() as u64;
    if ticks.0 == new_ticks { return; }
    ticks.0 = new_ticks;

    for (chunk, mesh_handle) in query.iter() {
        if chunk.has_animations {
            #[allow(clippy::collapsible_if)]
            if let Some((new_mesh, _)) = generate_chunk_mesh(&map_data, chunk.cx, chunk.cy, 32, ticks.0) {
                if let Some(mesh) = meshes.get_mut(mesh_handle.id()) {
                    *mesh = new_mesh;
                }
            }
        }
    }
}
