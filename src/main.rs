// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

// --- GAME CONSTANTS ---

/// The number of lanes available to the player.
const NUM_LANES: usize = 3;
/// The maximum lane index (0-based).
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
/// The x-coordinates for each lane.
const LANE_LOCATIONS: [f32; NUM_LANES] = [-2.5, 0.0, 2.5];
/// The delay between player inputs in seconds.
const INPUT_DELAY: f32 = 0.25;
/// The forward movement speed of the player and the world.
const SPEED: f32 = 15.0;
/// The strength of gravity affecting the player.
const GRAVITY: f32 = -30.0;
/// The initial upward velocity of the player's jump.
const JUMP_STRENGTH: f32 = 10.0;

// --- COMPONENTS ---

/// A marker component for the player entity.
#[derive(Component)]
struct Player;

/// A marker component for the ground plane entities.
#[derive(Component)]
struct Plane;

/// A marker component for obstacle entities.
#[derive(Component)]
struct Obstacle;

/// Stores the player's current lane index.
#[derive(Component)]
struct Lane {
    current: usize,
}

impl Default for Lane {
    fn default() -> Self {
        // Start the player in the middle lane.
        Self {
            current: NUM_LANES / 2,
        }
    }
}

/// A component to manage the delay between player inputs.
#[derive(Component)]
struct InputDelay {
    remaining: f32,
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// A resource to manage the spawning of obstacles.
#[derive(Resource)]
struct ObstacleSpawnTimer {
    timer: Timer,
}

/// Stores the player's horizontal movement speed.
#[derive(Component)]
struct LaneMovement {
    speed: f32,
}

impl Default for LaneMovement {
    fn default() -> Self {
        Self { speed: SPEED }
    }
}

/// Stores the player's vertical velocity for jumping and gravity.
#[derive(Component)]
struct VerticalMovement {
    velocity_y: f32,
}

impl Default for VerticalMovement {
    fn default() -> Self {
        Self { velocity_y: 0.0 }
    }
}

// --- MAIN FUNCTION ---

fn main() {
    App::new()
        // Add the default Bevy plugins, configuring the window.
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shupogaki 💢".into(),
                resolution: (1280.0, 720.0).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),))
        // Insert the obstacle spawn timer as a resource.
        .insert_resource(ObstacleSpawnTimer {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        // Add the setup system to run once at startup.
        .add_systems(Startup, setup)
        // Add the game's update systems to run every frame.
        .add_systems(
            Update,
            (
                handle_player_input,
                update_player_position,
                update_input_delay_time,
                apply_gravity_and_vertical_movement,
                update_plane_position,
                spawn_obstacles,
                update_obstacle_position,
                check_for_collisions,
            ),
        )
        // Run the Bevy application.
        .run();
}

// --- SETUP SYSTEM ---

/// A system that sets up the initial game world.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the ground planes.
    for i in 0..3 {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 50.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Srgba::rgb(0.3, 0.5, 0.3 + 0.1 * i as f32).into(),
                ..Default::default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0 + 50.0 * i as f32),
            Plane,
        ));
    }

    // Spawn the player cube.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.8, 0.7, 0.6).into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.5, -8.0),
        Lane::default(),
        InputDelay::default(),
        LaneMovement::default(),
        VerticalMovement::default(),
        Player,
    ));

    // Spawn a directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.0,
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn the 3D camera.
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scale: 1.25,
            scaling_mode: ScalingMode::Fixed {
                width: 16.0,
                height: 9.0,
            },
            near: 0.1,
            far: 100.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(12.0, 9.0, 12.0).looking_at((0.0, 1.5, 0.0).into(), Vec3::Y),
    ));
}

// --- UPDATE SYSTEMS ---

/// A type alias for the player input query data.
type PlayerInputData<'a> = (
    &'a Transform,
    &'a mut Lane,
    &'a mut InputDelay,
    &'a mut VerticalMovement,
);

/// A system that handles player input for movement and jumping.
fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<PlayerInputData, With<Player>>,
) {
    if let Ok((transform, mut lane, mut input_delay, mut vert_move)) = query.single_mut() {
        // Check if the input delay has passed and keys are not pressed simultaneously.
        if input_delay.remaining <= 0.0
            && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD])
        {
            // Move left.
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.current = lane.current.saturating_sub(1);
                input_delay.remaining = INPUT_DELAY;
            }
            // Move right.
            else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.current = lane.current.saturating_add(1).min(MAX_LANE_INDEX);
                input_delay.remaining = INPUT_DELAY;
            }
        }

        // Check if the player is on the ground.
        let is_grounded = transform.translation.y <= 0.5;
        // Jump if the space key is pressed and the player is on the ground.
        if keyboard_input.just_pressed(KeyCode::Space) && is_grounded {
            vert_move.velocity_y = JUMP_STRENGTH;
        }
    }
}

/// A system that smoothly updates the player's x-coordinate to match the current lane.
fn update_player_position(
    mut query: Query<(&mut Transform, &Lane), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, lane)) = query.single_mut() {
        // Calculate the target x-position based on the current lane.
        let target_x = LANE_LOCATIONS[lane.current];
        // Smoothly interpolate the player's x-position towards the target.
        transform.translation.x += (target_x - transform.translation.x) * SPEED * time.delta_secs();
    }
}

/// A system that decrements the input delay timer.
fn update_input_delay_time(mut query: Query<&mut InputDelay, With<Player>>, time: Res<Time>) {
    if let Ok(mut delay_time) = query.single_mut() {
        delay_time.remaining = (delay_time.remaining - time.delta_secs()).max(0.0);
    }
}

/// A system that applies gravity to the player and updates their vertical position.
fn apply_gravity_and_vertical_movement(
    mut query: Query<(&mut Transform, &mut VerticalMovement), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut vert_move)) = query.single_mut() {
        // Apply gravity to the vertical velocity.
        vert_move.velocity_y += GRAVITY * time.delta_secs();
        // Update the player's y-position based on the vertical velocity.
        transform.translation.y += vert_move.velocity_y * time.delta_secs();

        // Prevent the player from falling through the ground.
        if transform.translation.y < 0.5 {
            transform.translation.y = 0.5;
            vert_move.velocity_y = 0.0;
        }
    }
}

/// A system that moves the ground planes towards the player and recycles them.
fn update_plane_position(
    player_query: Query<&LaneMovement, With<Player>>,
    mut plane_query: Query<&mut Transform, With<Plane>>,
    time: Res<Time>,
) {
    if let Ok(lane_move) = player_query.single() {
        for mut transform in plane_query.iter_mut() {
            // Move the plane towards the player.
            transform.translation.z -= lane_move.speed * time.delta_secs();

            // If the plane is off-screen, move it to the back to create an infinite scrolling effect.
            if transform.translation.z < -50.0 {
                transform.translation.z += 150.0; // 3 planes * 50.0 length
            }
        }
    }
}

/// A system that spawns obstacles periodically.
fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_timer: ResMut<ObstacleSpawnTimer>,
    time: Res<Time>,
) {
    // Tick the spawn timer.
    spawn_timer.timer.tick(time.delta());

    // If the timer finished, spawn a new obstacle.
    if spawn_timer.timer.just_finished() {
        // Choose a random lane for the obstacle.
        let lane_index = rand::random_range(0..NUM_LANES);
        let lane_x = LANE_LOCATIONS[lane_index];

        // Spawn the obstacle entity.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            Transform::from_xyz(lane_x, 0.5, 100.0),
            Obstacle,
        ));
    }
}

/// A system that moves obstacles towards the player and despawns them when they are off-screen.
fn update_obstacle_position(
    mut commands: Commands,
    player_query: Query<&LaneMovement, With<Player>>,
    mut obstacle_query: Query<(Entity, &mut Transform), With<Obstacle>>,
    time: Res<Time>,
) {
    if let Ok(lane_move) = player_query.single() {
        for (entity, mut transform) in obstacle_query.iter_mut() {
            // Move the obstacle towards the player.
            transform.translation.z -= lane_move.speed * time.delta_secs();

            // If the obstacle is off-screen, despawn it.
            if transform.translation.z < -50.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// A system that checks for collisions between the player and obstacles.
fn check_for_collisions(
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
