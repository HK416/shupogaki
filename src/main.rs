use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::ScalingMode};

const NUM_LANES: usize = 3;
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
const LANE_LOCATIONS: [f32; NUM_LANES] = [-2.5, 0.0, 2.5];
const INPUT_DELAY: f32 = 0.25;
const SPEED: f32 = 15.0;
const GRAVITY: f32 = -30.0;
const JUMP_STRENGTH: f32 = 10.0;

/// A marker component for player entities.
#[derive(Component)]
struct Player;

/// A marker component for plane entities.
#[derive(Component)]
struct Plane;

/// Stores the player's current lane index.
#[derive(Component)]
struct Lane {
    current: usize,
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            current: NUM_LANES / 2, // Start in the middle lane.
        }
    }
}

/// Stores the remaining time for input delay.
#[derive(Component)]
struct InputDelay {
    remaining: f32,
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
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

fn main() {
    App::new()
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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_player_input,
                update_player_position,
                update_input_delay_time,
                apply_gravity_and_vertical_movement,
                update_plane_position,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
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

    // Cube Player
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.8, 0.7, 0.6).into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.5, 8.0),
        Lane::default(),
        InputDelay::default(),
        LaneMovement::default(),
        VerticalMovement::default(),
        Player,
    ));

    // Directional Light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
    ));

    // 3D camera using orthographic projection, looking slightly up.
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

type PlayerInputData<'a> = (
    &'a Transform,
    &'a mut Lane,
    &'a mut InputDelay,
    &'a mut VerticalMovement,
);

/// A system that moves the player left and right based on keyboard input
fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<PlayerInputData, With<Player>>,
) {
    if let Ok((transform, mut lane, mut input_delay, mut vert_move)) = query.single_mut() {
        if input_delay.remaining <= 0.0
            && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD])
        {
            // Press 'A' or the left arrow key to move left
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.current = lane.current.saturating_sub(1);
                input_delay.remaining = INPUT_DELAY;
            }
            // Press 'D' or the right arrow key to move right
            else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.current = lane.current.saturating_add(1).min(MAX_LANE_INDEX);
                input_delay.remaining = INPUT_DELAY;
            }
        }

        // Player can jump if they are on the ground.
        let is_grounded = transform.translation.y <= 0.5;
        if keyboard_input.just_pressed(KeyCode::Space) && is_grounded {
            vert_move.velocity_y = JUMP_STRENGTH;
        }
    }
}

/// A system that smoothly moves the player's current location to the target lane location.
fn update_player_position(
    mut query: Query<(&mut Transform, &Lane), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, lane)) = query.single_mut() {
        // Calculate target position.
        let target_x = LANE_LOCATIONS[lane.current];
        // Move smoothly.
        transform.translation.x += (target_x - transform.translation.x) * SPEED * time.delta_secs();
    }
}

/// A system that updates the remaining input delay time.
fn update_input_delay_time(mut query: Query<&mut InputDelay, With<Player>>, time: Res<Time>) {
    if let Ok(mut delay_time) = query.single_mut() {
        delay_time.remaining = (delay_time.remaining - time.delta_secs()).max(0.0);
    }
}

/// A system that applies gravity and updates the player's Y coordinate as a result.
fn apply_gravity_and_vertical_movement(
    mut query: Query<(&mut Transform, &mut VerticalMovement), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut vert_move)) = query.single_mut() {
        // Apply gravity.
        vert_move.velocity_y += GRAVITY * time.delta_secs();
        // Update vertical position based on velocity.
        transform.translation.y += vert_move.velocity_y * time.delta_secs();

        // Prevent falling through the ground (player height is 1.0, so ground is at y=0.5).
        if transform.translation.y < 0.5 {
            transform.translation.y = 0.5;
            vert_move.velocity_y = 0.0;
        }
    }
}

/// A system that moves the planes towards the player and recycles them.
fn update_plane_position(
    player_query: Query<&LaneMovement, With<Player>>,
    mut plane_query: Query<&mut Transform, With<Plane>>,
    time: Res<Time>,
) {
    if let Ok(lane_move) = player_query.single() {
        for mut transform in plane_query.iter_mut() {
            // Move the plane towards the player (negative Z direction in Bevy is forward).
            transform.translation.z -= lane_move.speed * time.delta_secs();

            // If the plane has moved past the player and is off-screen, recycle it.
            if transform.translation.z < -50.0 {
                // Move it back by the total length of all planes (3 * 50.0 = 150.0).
                transform.translation.z += 150.0;
            }
        }
    }
}
