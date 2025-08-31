mod asset;
mod collider;
mod scene;

// Import necessary Bevy modules.
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

// Conditionally import the Collider for debug gizmos.
#[cfg(not(feature = "no-debuging-gizmo"))]
use crate::collider::Collider;
// Import local modules for asset handling and game scenes.
use crate::{
    asset::spawner::CustomAssetPlugin,
    scene::{GameState, in_game, in_game_load},
};

// --- MAIN FUNCTION ---
// This is the entry point of the application.
fn main() {
    App::new()
        // --- CORE PLUGINS ---
        // Add the default Bevy plugins, which provide essential functionality.
        .add_plugins((DefaultPlugins
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
            }),))
        // --- CUSTOM PLUGINS ---
        // Add the custom asset plugin for loading and managing game assets.
        .add_plugins(CustomAssetPlugin)
        // --- STATE MANAGEMENT ---
        // Initialize the game's state machine. This controls the overall application flow,
        // determining which systems run based on the current state (e.g., InGameLoading vs. InGame).
        .init_state::<GameState>()
        // --- IN-GAME LOADING STATE ---
        // Define systems that run during the `InGameLoading` state.
        .add_systems(OnEnter(GameState::InGameLoading), in_game_load::on_enter)
        .add_systems(OnExit(GameState::InGameLoading), in_game_load::on_exit)
        .add_systems(
            Update,
            (
                // Checks the loading status of assets.
                in_game_load::check_loading_progress,
                // Updates the loading bar UI.
                in_game_load::update_loading_bar,
                // Animates the "Loading..." text.
                in_game_load::change_text_scale,
            )
                // This is a "Run Condition". These systems will only execute
                // if the game state is currently `InGameLoading`.
                .run_if(in_state(GameState::InGameLoading)),
        )
        // --- IN-GAME STATE ---
        // Define systems that run when entering the `InGame` state.
        .add_systems(
            OnEnter(GameState::InGame),
            (
                // Sets up the main game scene.
                in_game::on_enter,
                // Starts character and other animations.
                in_game::play_animation,
            ),
        )
        // Define systems that run when exiting the `InGame` state for cleanup.
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
                    (
                        // Update game logic like timers, score, and entity positions.
                        in_game::update_timer,
                        in_game::update_score,
                        in_game::update_fuel,
                        in_game::update_player_position,
                        in_game::update_ground_position,
                        in_game::update_obstacle_position,
                    )
                        // This is an "Ordering Constraint". It ensures that all game logic that
                        // moves entities runs *before* the collider positions are updated in the same frame.
                        // This is crucial for accurate, frame-perfect collision detection.
                        .before(in_game::update_collider),
                    // After all entities have their new positions, update the colliders to match.
                    in_game::update_collider,
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
                // Updates the toy train animation/position.
                in_game::update_toy_trains,
                // Spawns new ground segments as the player moves.
                in_game::spawn_grounds,
                // Spawns new obstacles.
                in_game::spawn_obstacles,
                // Checks for collisions between the player and obstacles.
                in_game::check_for_collisions,
                // Updates the score display on the UI.
                in_game::update_score_ui,
                in_game::update_fuel_deco,
                in_game::update_fuel_gauge,
                in_game::update_invincible_effect,
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
pub fn draw_collider_gizmos(mut gizmos: Gizmos, query: Query<&Collider>) {
    // Iterate over all entities with a Collider component.
    for collider in query.iter() {
        match collider {
            // If the collider is an AABB, draw a red cuboid.
            Collider::Aabb {
                center,
                size: extents,
            } => {
                gizmos.cuboid(
                    Transform::from_translation(*center).with_scale(*extents),
                    Color::srgb(1.0, 0.0, 0.0),
                );
            }
            // If the collider is a Sphere, draw a red sphere.
            Collider::Sphere { center, radius } => {
                gizmos
                    .sphere(
                        Isometry3d::from_translation(*center),
                        *radius,
                        Color::srgb(1.0, 0.0, 0.0),
                    )
                    // Increase the resolution for a smoother sphere.
                    .resolution(64);
            }
        };
    }
}
