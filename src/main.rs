//! The main entry point for the Shupogaki game application.
//!
//! This file is responsible for setting up the Bevy application, configuring plugins,
//! defining game states, and scheduling all the systems that make up the game's logic.

// `asset` module handles loading and management of custom game assets.
mod asset;
// `collider` module defines collider shapes and intersection logic.
mod collider;
// `scene` module contains the game's scenes, states, and core logic.
mod scene;

// Import necessary Bevy modules.
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_tweening::TweeningPlugin;

// Conditionally import the Collider for debug gizmos.
#[cfg(not(feature = "no-debuging-gizmo"))]
use crate::collider::Collider;
// Import local modules for asset handling and game scenes.
use crate::{
    asset::spawner::CustomAssetPlugin,
    scene::{GameState, in_game, loading, prepare},
};

// --- MAIN FUNCTION ---
// This is the entry point of the application.
fn main() {
    App::new()
        // --- CORE PLUGINS ---
        // Adds the default Bevy plugins, which provide essential cross-platform functionality.
        .add_plugins((
            DefaultPlugins
                // Configure the primary window.
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Shupogaki 💢".into(),
                        resolution: (1280.0, 720.0).into(),
                        // Fit the canvas to the parent element, useful for web builds.
                        fit_canvas_to_parent: true,
                        // Prevent the browser from handling default events, like scrolling.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                // Configure the logging plugin.
                .set(LogPlugin {
                    // Set the log level based on a feature flag.
                    // Use WARN level if 'no-debuging-log' is enabled, otherwise INFO.
                    level: if cfg!(feature = "no-debuging-log") {
                        Level::WARN
                    } else {
                        Level::INFO
                    },
                    ..Default::default()
                }),
            TweeningPlugin,
        ))
        // --- CUSTOM PLUGINS ---
        // Adds the custom asset plugin for loading and managing game assets.
        .add_plugins(CustomAssetPlugin)
        // --- STATE MANAGEMENT ---
        // Initializes the game's state machine. This controls the overall application flow,
        // determining which systems run based on the current state (e.g., Loading vs. InGame).
        .init_state::<GameState>()
        // --- LOADING STATE ---
        // Defines systems to run while assets are loading.
        .add_systems(OnEnter(GameState::Loading), loading::on_enter)
        .add_systems(OnExit(GameState::Loading), loading::on_exit)
        .add_systems(
            Update,
            (
                // Checks the loading progress of assets and transitions state when done.
                loading::check_loading_progress,
                // Updates the loading bar UI to reflect asset loading progress.
                loading::update_loading_bar,
                // Scales the "Loading..." text when the window is resized.
                loading::change_text_scale,
            )
                // This is a "Run Condition". These systems will only execute
                // if the game state is currently `GameState::Loading`.
                .run_if(in_state(GameState::Loading)),
        )
        // --- PREPARE STATE ---
        // Defines systems for the brief intro scene before gameplay starts.
        .add_systems(
            OnEnter(GameState::Prepare),
            (
                // Sets up the main game scene (player controller, camera, lighting).
                prepare::on_enter,
                // Starts character and other animations once their models are loaded.
                prepare::play_animation,
            ),
        )
        .add_systems(OnExit(GameState::Prepare), prepare::on_exit)
        .add_systems(
            Update,
            (
                // Advances the scene timer to transition to InGame state.
                prepare::update_scene_timer,
                // Animates the player moving into the start position.
                prepare::update_player_position.after(prepare::update_scene_timer),
                // Run background scrolling systems during the prepare state for a seamless transition.
                in_game::update_ground_position,
                in_game::update_object_position,
            )
                .run_if(in_state(GameState::Prepare)),
        )
        .add_systems(
            PostUpdate,
            (
                // These systems also run during Prepare to ensure the world is active and moving.
                in_game::update_toy_trains,
                in_game::spawn_grounds,
                in_game::spawn_objects,
                in_game::update_fuel_deco,
            )
                .run_if(in_state(GameState::Prepare)),
        )
        // --- IN-GAME STATE ---
        // Defines systems that run when the main gameplay starts.
        .add_systems(
            OnEnter(GameState::InGame),
            // Make the UI visible and start its animations.
            (in_game::on_enter, in_game::play_ui_animation),
        )
        // Defines systems that run when exiting the `InGame` state for cleanup.
        .add_systems(OnExit(GameState::InGame), in_game::on_exit)
        // --- GAMEPLAY SYSTEMS ---
        // Schedule systems to run at different stages of the game loop.
        .add_systems(
            PreUpdate,
            // Handle player input in the `PreUpdate` stage for responsiveness.
            (in_game::handle_player_input).run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (
                // Main game logic runs in the `Update` stage.
                (
                    // Update game logic like timers, score, and entity positions.
                    in_game::update_input_delay,
                    in_game::update_player_state,
                    in_game::update_score,
                    in_game::update_fuel,
                    in_game::update_player_position,
                    in_game::update_ground_position,
                    in_game::update_object_position,
                    in_game::cleanup_ui_animation,
                    // This system is for debugging player states and is only compiled when
                    // the "no-debuging-player" feature is NOT enabled.
                    #[cfg(not(feature = "no-debuging-player"))]
                    {
                        in_game::handle_player
                    },
                )
                    .run_if(in_state(GameState::InGame)),
                // This is a "conditional compilation" attribute. The code below will only be included
                // in the build if the "no-debuging-gizmo" feature flag is NOT enabled.
                // This is useful for excluding debug-only code from release builds.
                #[cfg(not(feature = "no-debuging-gizmo"))]
                {
                    (update_gizmo_config, draw_collider_gizmos)
                },
            ),
        )
        .add_systems(
            PostUpdate,
            // Systems that react to changes from the `Update` stage.
            (
                in_game::update_toy_trains,
                in_game::spawn_grounds,
                in_game::spawn_objects,
                in_game::check_for_collisions,
                in_game::update_score_ui,
                in_game::update_fuel_deco,
                in_game::update_fuel_gauge,
                in_game::update_player_effect,
                in_game::update_player_speed,
            )
                .run_if(in_state(GameState::InGame)),
        )
        // --- RUN THE APP ---
        // Start the Bevy application loop.
        .run();
}

// --- DEBUG GIZMO SYSTEMS ---
// These systems are only compiled if the "no-debuging-gizmo" feature is NOT enabled.

/// Toggles the visibility of debug gizmos when the F4 key is pressed.
// This system is only compiled if the "no-debuging-gizmo" feature is NOT enabled.
#[cfg(not(feature = "no-debuging-gizmo"))]
pub fn update_gizmo_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Check if F4 was just pressed.
    if keyboard_input.just_pressed(KeyCode::F4) {
        // Iterate through all gizmo configurations and toggle their `enabled` flag.
        for (_, config, _) in config_store.iter_mut() {
            // `^=` is the XOR assignment operator, a concise way to toggle a boolean.
            config.enabled ^= true;
        }
    }
}

/// Draws visual representations (gizmos) for all `Collider` components in the scene.
// This system is only compiled if the "no-debuging-gizmo" feature is NOT enabled.
#[cfg(not(feature = "no-debuging-gizmo"))]
pub fn draw_collider_gizmos(mut gizmos: Gizmos, query: Query<(&Collider, &Transform)>) {
    const GIZMO_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);

    // Iterate over all entities with a Collider component.
    for (collider, transform) in query.iter() {
        // Draw axes to show the orientation of the entity.
        gizmos.axes(*transform, 2.0);

        match collider {
            Collider::Aabb { offset, size } => {
                let center = transform.translation + *offset;
                gizmos.cuboid(
                    Transform::from_translation(center).with_scale(*size),
                    GIZMO_COLOR,
                );
            }
            Collider::Sphere { offset, radius } => {
                let center = transform.translation + *offset;
                gizmos.sphere(Isometry3d::from_translation(center), *radius, GIZMO_COLOR);
            }
        }
    }
}
