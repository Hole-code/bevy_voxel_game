use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use noise::{NoiseFn, Perlin};

const CHUNK_SIZE: i32 = 16;
const WORLD_SIZE: i32 = 4;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Chunk {
    position: IVec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, generate_chunks))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Player,
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Generate initial chunks
    for x in -WORLD_SIZE..WORLD_SIZE {
        for z in -WORLD_SIZE..WORLD_SIZE {
            spawn_chunk(&mut commands, &mut meshes, &mut materials, IVec3::new(x, 0, z));
        }
    }
}

fn player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if input.pressed(KeyCode::W) {
        direction += player_transform.forward();
    }
    if input.pressed(KeyCode::S) {
        direction += player_transform.back();
    }
    if input.pressed(KeyCode::A) {
        direction += player_transform.left();
    }
    if input.pressed(KeyCode::D) {
        direction += player_transform.right();
    }
    if input.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if input.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    let speed = 10.0;
    player_transform.translation += direction.normalize_or_zero() * speed * time.delta_seconds();
}

fn generate_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<(Entity, &Chunk)>,
) {
    let player_transform = player_query.single();
    let player_chunk = (player_transform.translation / (CHUNK_SIZE as f32)).as_ivec3();

    // Despawn chunks that are too far
    for (entity, chunk) in chunk_query.iter() {
        if (chunk.position - player_chunk).abs().max_element() > WORLD_SIZE {
            commands.entity(entity).despawn();
        }
    }

    // Spawn new chunks
    for x in -WORLD_SIZE..=WORLD_SIZE {
        for z in -WORLD_SIZE..=WORLD_SIZE {
            let chunk_pos = player_chunk + IVec3::new(x, 0, z);
            if !chunk_query.iter().any(|(_, c)| c.position == chunk_pos) {
                spawn_chunk(&mut commands, &mut meshes, &mut materials, chunk_pos);
            }
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: IVec3,
) {
    let noise = Perlin::new(0);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let world_x = position.x * CHUNK_SIZE + x;
            let world_z = position.z * CHUNK_SIZE + z;
            let height = (noise.get([world_x as f64 * 0.02, world_z as f64 * 0.02]) * 10.0 + 10.0) as i32;

            for y in 0..height {
                add_cube(&mut vertices, &mut indices, &mut normals, x, y, z);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
            transform: Transform::from_xyz(
                (position.x * CHUNK_SIZE) as f32,
                0.0,
                (position.z * CHUNK_SIZE) as f32,
            ),
            ..default()
        },
        Chunk { position },
    ));
}

fn add_cube(vertices: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>, normals: &mut Vec<[f32; 3]>, x: i32, y: i32, z: i32) {
    let x = x as f32;
    let y = y as f32;
    let z = z as f32;
    let v_index = vertices.len() as u32;

    // Vertices
    vertices.extend_from_slice(&[
        [x, y, z],
        [x + 1.0, y, z],
        [x + 1.0, y + 1.0, z],
        [x, y + 1.0, z],
        [x, y, z + 1.0],
        [x + 1.0, y, z + 1.0],
        [x + 1.0, y + 1.0, z + 1.0],
        [x, y + 1.0, z + 1.0],
    ]);

    // Indices
    indices.extend_from_slice(&[
        v_index, v_index + 1, v_index + 2, v_index, v_index + 2, v_index + 3, // Front
        v_index + 5, v_index + 4, v_index + 7, v_index + 5, v_index + 7, v_index + 6, // Back
        v_index + 4, v_index, v_index + 3, v_index + 4, v_index + 3, v_index + 7, // Left
        v_index + 1, v_index + 5, v_index + 6, v_index + 1, v_index + 6, v_index + 2, // Right
        v_index + 3, v_index + 2, v_index + 6, v_index + 3, v_index + 6, v_index + 7, // Top
        v_index + 4, v_index + 5, v_index + 1, v_index + 4, v_index + 1, v_index, // Bottom
    ]);

    // Normals
    normals.extend_from_slice(&[
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ]);
}