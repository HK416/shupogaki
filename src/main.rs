use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::ScalingMode};

const NUM_LANES: usize = 3;
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
const LANE_LOCATIONS: [f32; NUM_LANES] = [-2.5, 0.0, 2.5];
const INPUT_DELAY: f32 = 0.25;
const SPEED: f32 = 15.0;

/// identifying player entities.
#[derive(Component)]
struct Player;

/// lane where the player is located.
#[derive(Component)]
struct Lane {
    current: usize,
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            current: NUM_LANES / 2,
        }
    }
}

/// remaining input delay time.
#[derive(Component)]
struct InputDelay {
    remaining: f32,
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
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
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.3, 0.5, 0.3).into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

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

    // 3D camera using orthographic projection
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
        Transform::from_xyz(12.0, 9.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// A system that moves the player left and right based on keyboard input
fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Lane, &mut InputDelay), With<Player>>,
) {
    if let Ok((mut lane, mut input_delay)) = query.single_mut() {
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
