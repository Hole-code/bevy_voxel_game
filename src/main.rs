use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

const CHUNK_SIZE: i32 = 16;
const RENDER_DISTANCE: i32 = 3;

#[derive(Component)]
struct Chunk {
    position: IVec3,
}

#[derive(Component)]
struct Player;

#[derive(Resource, Default)]
struct WorldMap {
    chunks: HashMap<IVec3, Vec<Vec<Vec<bool>>>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window>,
) {
    // Камера и игрок
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 0.0),
            ..default()
        },
        Player,
    ));

    // Свет
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Инициализация мира
    let world_map = WorldMap {
        chunks: HashMap::new(),
    };
    commands.insert_resource(world_map);

    // Настройка захвата курсора
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
}

fn generate_chunk(chunk_pos: IVec3) -> Vec<Vec<Vec<bool>>> {
    let mut chunk = vec![vec![vec![false; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
    let perlin = Perlin::new(0);
    
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let world_x = chunk_pos.x * CHUNK_SIZE + x;
            let world_z = chunk_pos.z * CHUNK_SIZE + z;
            let height = (perlin.get([world_x as f64 * 0.01, world_z as f64 * 0.01]) * 32.0 + 32.0) as i32;
            
            for y in 0..CHUNK_SIZE {
                let world_y = chunk_pos.y * CHUNK_SIZE + y;
                if world_y < height {
                    chunk[x as usize][y as usize][z as usize] = true;
                }
            }
        }
    }
    
    chunk
}

fn update_visible_chunks(
    mut commands: Commands,
    mut world_map: ResMut<WorldMap>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<(Entity, &Chunk)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_transform = player_query.single();
    let player_chunk = IVec3::new(
        (player_transform.translation.x / (CHUNK_SIZE as f32)).floor() as i32,
        (player_transform.translation.y / (CHUNK_SIZE as f32)).floor() as i32,
        (player_transform.translation.z / (CHUNK_SIZE as f32)).floor() as i32,
    );

    let mut chunks_to_remove = Vec::new();
    for (entity, chunk) in chunk_query.iter() {
        if (chunk.position - player_chunk).abs().max_element() > RENDER_DISTANCE {
            chunks_to_remove.push(entity);
        }
    }
    for entity in chunks_to_remove {
        commands.entity(entity).despawn();
    }

    for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for y in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = player_chunk + IVec3::new(x, y, z);
                if !world_map.chunks.contains_key(&chunk_pos) {
                    let chunk_data = generate_chunk(chunk_pos);
                    world_map.chunks.insert(chunk_pos, chunk_data.clone());
                    spawn_chunk(&mut commands, &mut meshes, &mut materials, chunk_pos, &chunk_data);
                }
            }
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_pos: IVec3,
    chunk_data: &Vec<Vec<Vec<bool>>>,
) {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if chunk_data[x as usize][y as usize][z as usize] {
                    add_cube_to_mesh(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        Vec3::new(x as f32, y as f32, z as f32),
                    );
                }
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(
                chunk_pos.x as f32 * CHUNK_SIZE as f32,
                chunk_pos.y as f32 * CHUNK_SIZE as f32,
                chunk_pos.z as f32 * CHUNK_SIZE as f32,
            ),
            ..default()
        },
        Chunk { position: chunk_pos },
    ));
}

fn add_cube_to_mesh(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    position: Vec3,
) {
    let (x, y, z) = (position.x, position.y, position.z);
    let cube_vertices = [
        [x, y, z], [x+1.0, y, z], [x+1.0, y+1.0, z], [x, y+1.0, z],
        [x, y, z+1.0], [x+1.0, y, z+1.0], [x+1.0, y+1.0, z+1.0], [x, y+1.0, z+1.0],
    ];
    let cube_indices = [
        0, 1, 2, 2, 3, 0, // Front face
        1, 5, 6, 6, 2, 1, // Right face
        5, 4, 7, 7, 6, 5, // Back face
        4, 0, 3, 3, 7, 4, // Left face
        3, 2, 6, 6, 7, 3, // Top face
        4, 5, 1, 1, 0, 4, // Bottom face
    ];
    let cube_normals = [
        [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0],
    ];

    let start_index = vertices.len() as u32;
    vertices.extend_from_slice(&cube_vertices);
    indices.extend(cube_indices.iter().map(|&i| i + start_index));
    normals.extend(cube_normals.iter().flat_map(|&n| std::iter::repeat(n).take(4)));
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    world_map: Res<WorldMap>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::W) {
        direction += player_transform.forward();
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += player_transform.back();
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction += player_transform.left();
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += player_transform.right();
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    let new_position = player_transform.translation + direction * 5.0 * time.delta_seconds();
    
    // Проверка коллизий
    if !check_collision(&world_map, new_position) {
        player_transform.translation = new_position;
    }
}

fn check_collision(world_map: &WorldMap, position: Vec3) -> bool {
    let chunk_pos = IVec3::new(
        (position.x / CHUNK_SIZE as f32).floor() as i32,
        (position.y / CHUNK_SIZE as f32).floor() as i32,
        (position.z / CHUNK_SIZE as f32).floor() as i32,
    );

    if let Some(chunk) = world_map.chunks.get(&chunk_pos) {
        let local_pos = IVec3::new(
            (position.x.rem_euclid(CHUNK_SIZE as f32)) as i32,
            (position.y.rem_euclid(CHUNK_SIZE as f32)) as i32,
            (position.z.rem_euclid(CHUNK_SIZE as f32)) as i32,
        );
        chunk[local_pos.x as usize][local_pos.y as usize][local_pos.z as usize]
    } else {
        false
    }
}

fn player_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    for event in mouse_motion_events.read() {
        player_transform.rotate_y(-event.delta.x * 0.002);
        player_transform.rotate_local_x(-event.delta.y * 0.002);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<WorldMap>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_movement,
            player_look,
            update_visible_chunks,
        ))
        .run();
}