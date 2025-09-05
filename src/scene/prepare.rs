// src/scene/prepare.rs

//! This module handles the `Prepare` game state, which serves as a brief
//! introductory scene before the main gameplay begins. It is responsible for:
//! - Initializing all necessary in-game resources.
//! - Spawning the player, camera, and lighting.
//! - Playing a short animation where the player moves into the starting position.
//! - Transitioning to the `InGame` state after a fixed duration.

// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use super::*;

// --- CONSTANTS ---

/// The duration of the preparation scene in seconds.
const SCENE_DURATION: f32 = 3.0;

// --- SETUP SYSTEM ---

/// A system that runs once when entering `GameState::Prepare`.
/// It initializes all necessary resources and spawns the core entities for the game.
pub fn on_setup(mut commands: Commands) {
    info!("Enter Prepare state.");
    // --- Resource initialization ---
    // Insert resources required for the InGame state.
    commands.insert_resource(TrainFuel::default());
    commands.insert_resource(InputDelay::default());
    commands.insert_resource(PlayScore::default());
    commands.insert_resource(PlayerState::default());
    commands.insert_resource(RetiredGrounds::default());
    commands.insert_resource(ObjectSpawner::default());
    commands.insert_resource(SceneTimer::default());

    // --- Player Spawn ---
    // Spawn the main player controller entity. This entity itself is invisible
    // but holds the core movement logic, collider, and other essential components.
    commands.spawn((
        Transform::from_xyz(LANE_LOCATIONS[1], 0.0, PLAYER_MAX_Z_POS),
        Lane::default(),
        ForwardMovement::default(),
        VerticalMovement::default(),
        Collider::Aabb {
            offset: Vec3::new(0.0, 0.5, -1.5),
            size: Vec3::new(0.9, 1.0, 3.6),
        },
        InGameStateEntity, // Marker for game-specific entities.
        Player,            // Marker for the player entity.
    ));

    // --- Lighting ---
    // Spawn a directional light to illuminate the scene.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0, // A bright, sun-like light.
            shadows_enabled: true, // Enable shadows for realism.
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        InGameStateEntity,
    ));

    // --- Camera Spawn ---
    // Spawn the 3D camera with an orthographic projection for a stylized, fixed-perspective look.
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            near: 0.1,
            far: 100.0,
            // Use a fixed scaling mode to ensure the view is consistent across different window sizes.
            scaling_mode: ScalingMode::Fixed {
                width: 16.0,
                height: 9.0,
            },
            scale: 1.25, // Zoom out slightly.
            ..OrthographicProjection::default_3d()
        }),
        // Position the camera and make it look at a point slightly above the origin.
        Transform::from_xyz(12.0, 9.0, 12.0).looking_at((0.0, 1.5, 0.0).into(), Vec3::Y),
        InGameStateEntity,
    ));
}

/// A system that plays the animation for entities with an `AnimationClipHandle`.
/// This system is separate from the entity spawning to ensure that the animation graph is correctly setup after the model has been loaded.
pub fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<(Entity, &AnimationClipHandle), Without<ResultStateEntity>>,
) {
    for (entity, clip) in query.iter() {
        let (graph, animation_index) = AnimationGraph::from_clip(clip.0.clone());
        let mut player = AnimationPlayer::default();
        player.play(animation_index).repeat();

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player))
            .remove::<AnimationClipHandle>();
    }
}

/// A system that makes all entities marked with `InGameStateEntity` (excluding UI elements)
/// visible. This is typically used after pre-spawning hidden entities.
pub fn visible_in_game_entities(
    mut query: Query<&mut Visibility, (With<InGameStateEntity>, Without<UI>)>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

// --- CLEANUP SYSTEM ---

/// A system that runs once when exiting `GameState::Prepare`.
/// It cleans up resources specific to this state.
pub fn on_exit(mut commands: Commands) {
    info!("Exit Prepare state.");
    commands.remove_resource::<SceneTimer>();
}

// --- UPDATE SYSTEMS ---

/// A system that advances the scene timer and transitions to the `InGame` state when it finishes.
pub fn update_scene_timer(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta_secs());
    if timer.0 >= SCENE_DURATION {
        next_state.set(GameState::InGame);
    }
}

// --- POSTUPDATE SYSTEMS ---

/// A system that animates the player's position during the intro sequence.
pub fn update_player_position(
    mut query: Query<&mut Transform, With<Player>>,
    timer: Res<SceneTimer>,
) {
    if let Ok(mut transform) = query.single_mut() {
        // Calculate the interpolation factor `t` from 0.0 to 1.0 based on the scene timer.
        let t = timer.0 / SCENE_DURATION;
        // Linearly interpolate the player's z-position from the starting point to the final gameplay position.
        let z_pos = PLAYER_MIN_Z_POS * (1.0 - t) + PLAYER_MAX_Z_POS * t;
        transform.translation.z = z_pos;
    }
}
