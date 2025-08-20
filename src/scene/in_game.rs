use std::collections::{HashMap, VecDeque};

// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use crate::asset::spawner::SpawnModel;

// --- GAME CONSTANTS ---

/// The number of lanes available to the player.
const NUM_LANES: usize = 3;
/// The maximum lane index (0-based).
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
/// The x-coordinates for each lane.
const LANE_LOCATIONS: [f32; NUM_LANES] = [-3.0, 0.0, 3.0];
/// The delay between player inputs in seconds.
const INPUT_DELAY: f32 = 0.25;
/// The delay between obstacle creation in seconds.
const SPAWN_DELAY: f32 = 2.0;
/// The forward movement speed of the player and the world.
const SPEED: f32 = 20.0;
/// The strength of gravity affecting the player.
const GRAVITY: f32 = -30.0;
/// The initial upward velocity of the player's jump.
const JUMP_STRENGTH: f32 = 12.5;
// The lane change speed of the player.
const LANE_CHANGE_SPEED: f32 = 5.0;

// --- COMPONENTS ---

/// A marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// A marker component for the ground plane entities.
#[derive(Component)]
pub struct Ground;

/// A marker component for obstacle entities.
#[derive(Component)]
pub struct Obstacle;

/// A marker component for in_game state entities.
#[derive(Component)]
pub struct InGameStateEntity;

/// A marker component for first toy train entity.
#[derive(Component)]
pub struct ToyTrain0;

/// A marker component for seconds toy train entity.
#[derive(Component)]
pub struct ToyTrain1;

/// A marker component for third toy train entity.
#[derive(Component)]
pub struct ToyTrain2;

/// Stores the player's current lane index.
#[derive(Component)]
pub struct Lane {
    index: usize,
}

impl Default for Lane {
    fn default() -> Self {
        // Start the player in the middle lane.
        Self {
            index: NUM_LANES / 2,
        }
    }
}

/// Stores the player's horizontal movement speed.
#[derive(Component)]
pub struct ForwardMovement {
    velocity: f32,
}

impl Default for ForwardMovement {
    fn default() -> Self {
        Self { velocity: SPEED }
    }
}

/// Stores the player's vertical movement speed for jumping and gravity.
#[derive(Component)]
pub struct VerticalMovement {
    velocity: f32,
}

impl Default for VerticalMovement {
    fn default() -> Self {
        Self { velocity: 0.0 }
    }
}

// --- RESOURCES ---

/// A resource to manage the delay between player inputs.
#[derive(Resource)]
pub struct InputDelay {
    remaining: f32,
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// A resource to manage the spawning of obstacles.
#[derive(Resource)]
pub struct ObstacleSpawnTimer {
    remaining: f32,
}

impl Default for ObstacleSpawnTimer {
    fn default() -> Self {
        Self {
            remaining: SPAWN_DELAY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroundModel {
    Test0,
    Test1,
    Test2,
}

#[derive(Default, Resource)]
pub struct GroundModels {
    meshes: HashMap<GroundModel, Handle<Mesh>>,
    materials: HashMap<GroundModel, Handle<StandardMaterial>>,
}

#[derive(Default, Resource)]
pub struct RetiredGrounds {
    transforms: VecDeque<Transform>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ObstacleModel {
    Test,
}

#[derive(Default, Resource)]
pub struct ObstacleModels {
    meshes: HashMap<ObstacleModel, Handle<Mesh>>,
    materials: HashMap<ObstacleModel, Handle<StandardMaterial>>,
}

// --- SETUP SYSTEM ---

/// A system that sets up the initial game world.
pub fn on_enter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert resources
    commands.insert_resource(InputDelay::default());
    commands.insert_resource(ObstacleSpawnTimer::default());

    // Create and insert ground models resource
    let mesh_handle = meshes.add(Plane3d::default().mesh().size(10.0, 50.0));
    let material_handle_0 = materials.add(StandardMaterial {
        base_color: Srgba::rgb(0.3, 0.5, 0.3).into(),
        ..Default::default()
    });
    let material_handle_1 = materials.add(StandardMaterial {
        base_color: Srgba::rgb(0.3, 0.5, 0.4).into(),
        ..Default::default()
    });
    let material_handle_2 = materials.add(StandardMaterial {
        base_color: Srgba::rgb(0.3, 0.5, 0.5).into(),
        ..Default::default()
    });

    let mut ground_models = GroundModels::default();
    ground_models
        .meshes
        .insert(GroundModel::Test0, mesh_handle.clone());
    ground_models
        .materials
        .insert(GroundModel::Test0, material_handle_0.clone());
    ground_models
        .meshes
        .insert(GroundModel::Test1, mesh_handle.clone());
    ground_models
        .materials
        .insert(GroundModel::Test1, material_handle_1.clone());
    ground_models
        .meshes
        .insert(GroundModel::Test2, mesh_handle.clone());
    ground_models
        .materials
        .insert(GroundModel::Test2, material_handle_2.clone());

    commands.insert_resource(ground_models);
    commands.insert_resource(RetiredGrounds::default());

    // Spawn initial ground entities
    for i in 0..3 {
        let material = match i % 3 {
            0 => material_handle_0.clone(),
            1 => material_handle_1.clone(),
            2 => material_handle_2.clone(),
            _ => unreachable!(),
        };
        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, 50.0 * i as f32),
            Ground,
            InGameStateEntity,
        ));
    }

    // Create and insert obstacle models resource
    let mut obstacle_models = ObstacleModels::default();
    let mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });

    obstacle_models
        .meshes
        .insert(ObstacleModel::Test, mesh_handle);
    obstacle_models
        .materials
        .insert(ObstacleModel::Test, material_handle);

    commands.insert_resource(obstacle_models);

    // Spawn the player model from a custom asset.
    commands.spawn((
        SpawnModel(asset_server.load("./models/ToyTrain00.hierarchy")),
        Transform::default(),
        InGameStateEntity,
        ToyTrain0,
    ));
    let entity = commands
        .spawn((
            SpawnModel(asset_server.load("./models/CH0242.hierarchy")),
            Transform::from_xyz(0.0, 0.8775, 0.0),
            InGameStateEntity,
        ))
        .id();
    commands
        .spawn((
            SpawnModel(asset_server.load("./models/ToyTrain01.hierarchy")),
            Transform::default(),
            InGameStateEntity,
            ToyTrain1,
        ))
        .add_child(entity);
    let entity = commands
        .spawn((
            SpawnModel(asset_server.load("./models/CH0243.hierarchy")),
            Transform::from_xyz(0.0, 0.375, 0.375),
            InGameStateEntity,
        ))
        .id();
    commands
        .spawn((
            SpawnModel(asset_server.load("./models/ToyTrain02.hierarchy")),
            Transform::default(),
            InGameStateEntity,
            ToyTrain2,
        ))
        .add_child(entity);
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, -7.5),
        Lane::default(),
        ForwardMovement::default(),
        VerticalMovement::default(),
        InGameStateEntity,
        Player,
    ));

    // Spawn the player cube.
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Srgba::rgb(0.8, 0.7, 0.6).into(),
    //         ..Default::default()
    //     })),
    //     Transform::from_xyz(0.0, 0.5, -8.0),
    //     Lane::default(),
    //     ForwardMovement::default(),
    //     VerticalMovement::default(),
    //     InGameStateEntity,
    //     Player,
    // ));

    // Spawn a directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        InGameStateEntity,
    ));

    // Spawn the 3D camera.
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            near: 0.1,
            far: 100.0,
            scaling_mode: ScalingMode::Fixed {
                width: 16.0,
                height: 9.0,
            },
            scale: 1.25,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(12.0, 9.0, 12.0).looking_at((0.0, 1.5, 0.0).into(), Vec3::Y),
        InGameStateEntity,
    ));
}

// --- CLEANUP SYSTEM ---

/// A system that cleans up the game world when exiting the state.
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<InGameStateEntity>>) {
    // Despawn all entities associated with the InGame state.
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove resources specific to the InGame state.
    commands.remove_resource::<ObstacleModels>();
    commands.remove_resource::<RetiredGrounds>();
    commands.remove_resource::<GroundModels>();
    commands.remove_resource::<ObstacleSpawnTimer>();
    commands.remove_resource::<InputDelay>();
}

// --- PREUPDATE SYSTEMS ---

/// A system that handles player input for movement and jumping.
pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Lane, &mut VerticalMovement), With<Player>>,
    mut delay: ResMut<InputDelay>,
) {
    if let Ok((transform, mut lane, mut vert_move)) = query.single_mut() {
        // Check if the input delay has passed and keys are not pressed simultaneously.
        if delay.remaining <= 0.0 && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD]) {
            // Move left.
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.index = lane.index.saturating_sub(1);
                delay.remaining = INPUT_DELAY;
            }
            // Move right.
            else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.index = lane.index.saturating_add(1).min(MAX_LANE_INDEX);
                delay.remaining = INPUT_DELAY;
            }
        }

        // Check if the player is on the ground.
        let is_grounded = transform.translation.y <= 0.0;
        // Jump if the space key is pressed and the player is on the ground.
        if keyboard_input.just_pressed(KeyCode::Space) && is_grounded {
            vert_move.velocity = JUMP_STRENGTH;
        }
    }
}

// --- UPDATE SYSTEMS ---

/// A system that decrements timers for input delay and obstacle spawning.
pub fn update_timer(
    mut delay: ResMut<InputDelay>,
    mut spawn_timer: ResMut<ObstacleSpawnTimer>,
    time: Res<Time>,
) {
    delay.remaining = (delay.remaining - time.delta_secs()).max(0.0);
    spawn_timer.remaining -= time.delta_secs();
}

/// A system that smoothly updates the player's position based on lane and vertical movement.
pub fn update_player_position(
    mut query: Query<(&mut Transform, &Lane, &mut VerticalMovement), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, lane, mut vert_move)) = query.single_mut() {
        // Calculate the target x-position based on the current lane.
        let target_x = LANE_LOCATIONS[lane.index];
        // Smoothly interpolate the player's x-position towards the target.
        transform.translation.x +=
            (target_x - transform.translation.x) * LANE_CHANGE_SPEED * time.delta_secs();

        // Apply gravity to the vertical velocity.
        vert_move.velocity += GRAVITY * time.delta_secs();
        // Update the player's y-position based on the vertical velocity.
        transform.translation.y += vert_move.velocity * time.delta_secs();

        // Prevent the player from falling through the ground.
        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            vert_move.velocity = 0.0;
        }
    }
}

/// A system that moves the ground planes towards the player and recycles them.
pub fn update_ground_position(
    mut commands: Commands,
    player_query: Query<&ForwardMovement, With<Player>>,
    mut ground_query: Query<(Entity, &mut Transform), With<Ground>>,
    mut retired: ResMut<RetiredGrounds>,
    time: Res<Time>,
) {
    if let Ok(forward_move) = player_query.single() {
        for (entity, mut transform) in ground_query.iter_mut() {
            // Move the ground towards the player.
            transform.translation.z -= forward_move.velocity * time.delta_secs();

            // If the ground is off-screen, despawn it and add its transform to the retired queue.
            if transform.translation.z <= -50.0 {
                retired.transforms.push_back(*transform);
                commands.entity(entity).despawn();
            }
        }
    }
}

/// A system that moves obstacles towards the player and despawns them when they are off-screen.
pub fn update_obstacle_position(
    mut commands: Commands,
    player_query: Query<&ForwardMovement, With<Player>>,
    mut obstacle_query: Query<(Entity, &mut Transform), With<Obstacle>>,
    time: Res<Time>,
) {
    if let Ok(forward_move) = player_query.single() {
        for (entity, mut transform) in obstacle_query.iter_mut() {
            // Move the obstacle towards the player.
            transform.translation.z -= forward_move.velocity * time.delta_secs();

            // If the obstacle is off-screen, despawn it.
            if transform.translation.z <= -50.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

/// A system that updates the positions and rotations of the toy train cars.
///
/// This system creates a chain-like movement where each train car follows the one in front of it.
/// The first car follows the player's invisible controller entity.
pub fn update_toy_trains(
    mut set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<ToyTrain0>>,
        Query<&mut Transform, With<ToyTrain1>>,
        Query<&mut Transform, With<ToyTrain2>>,
    )>,
) {
    // Get the player's target position, which is slightly ahead of the player's actual position.
    let data = set
        .p0()
        .single()
        .map(|transform| transform.translation.with_z(transform.translation.z + 1.5))
        .ok();

    if let Some(mut position) = data {
        // Update the first train car.
        if let Ok(mut transform) = set.p1().single_mut() {
            // Calculate the rotation to make the car look at the target position.
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            // Store the current position to be used as the target for the next car.
            let temp = transform.translation;
            // Move the car to the target position.
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -7.5;
            transform.rotation = rotation;
            // Update the target position for the next car.
            position = temp;
        }

        // Update the second train car.
        if let Ok(mut transform) = set.p2().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            let temp = transform.translation;
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -9.0;
            transform.rotation = rotation;
            position = temp;
        }

        // Update the third train car.
        if let Ok(mut transform) = set.p3().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -10.5;
            transform.rotation = rotation;
        }
    }
}

/// A system that spawns new ground entities to create an infinite scrolling effect.
pub fn spawn_grounds(
    mut commands: Commands,
    mut retired: ResMut<RetiredGrounds>,
    models: Res<GroundModels>,
) {
    const MODELS: [GroundModel; 3] = [GroundModel::Test0, GroundModel::Test1, GroundModel::Test2];

    while let Some(mut transform) = retired.transforms.pop_front() {
        // Use thread_rng for better random number generation.
        let model = MODELS[rand::random_range(0..MODELS.len())];
        let mesh_handle = models.meshes.get(&model).cloned().unwrap();
        let material_handle = models.materials.get(&model).cloned().unwrap();
        transform.translation.z += 150.0;

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            transform,
            Ground,
            InGameStateEntity,
        ));
    }
}

/// A system that spawns obstacles periodically.
pub fn spawn_obstacles(
    mut commands: Commands,
    mut spawn_timer: ResMut<ObstacleSpawnTimer>,
    player_query: Query<&ForwardMovement, With<Player>>,
    models: Res<ObstacleModels>,
) {
    if let Ok(forward_move) = player_query.single() {
        while spawn_timer.remaining <= 0.0 {
            let time_t = -spawn_timer.remaining;

            // Choose a random lane for the obstacle.
            let lane_index = rand::random_range(0..NUM_LANES);
            let lane_x = LANE_LOCATIONS[lane_index];

            let mesh_handle = models.meshes.get(&ObstacleModel::Test).cloned().unwrap();
            let material_handle = models.materials.get(&ObstacleModel::Test).cloned().unwrap();

            // Spawn the obstacle entity.
            commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(lane_x, 0.5, 100.0 - forward_move.velocity * time_t),
                Obstacle,
                InGameStateEntity,
            ));

            spawn_timer.remaining += SPAWN_DELAY;
        }
    }
}

/// A system that checks for collisions between the player and obstacles.
pub fn check_for_collisions(
    player_query: Query<&Transform, (With<Player>, Without<Obstacle>)>,
    obstacle_query: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        for obstacle_transform in obstacle_query.iter() {
            // Calculate the distance between the player and the obstacle.
            let distance = player_transform
                .translation
                .distance(obstacle_transform.translation);

            // A simple collision check based on distance.
            if distance < 1.0 {
                println!("GAME OVER!");
                // In a real game, you would transition to a game over state here.
            }
        }
    }
}
