mod asset;
mod scene;

// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::{
    asset::spawner::CustomAssetPlugin,
    scene::{GameState, in_game},
};

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
        // Add the custom asset plugin.
        .add_plugins(CustomAssetPlugin)
        // Initialize the game state.
        .init_state::<GameState>()
        // Add systems that run when entering the InGame state.
        .add_systems(OnEnter(GameState::InGame), in_game::on_enter)
        // Add systems that run when exiting the InGame state.
        .add_systems(OnExit(GameState::InGame), in_game::on_exit)
        // Add systems that run in the PreUpdate stage.
        .add_systems(
            PreUpdate,
            (in_game::handle_player_input).run_if(in_state(GameState::InGame)),
        )
        // Add systems that run in the Update stage.
        .add_systems(
            Update,
            (
                in_game::update_timer,
                in_game::update_player_position,
                in_game::update_ground_position,
                in_game::update_obstacle_position,
            )
                .run_if(in_state(GameState::InGame)),
        )
        // Add systems that run in the PostUpdate stage.
        .add_systems(
            PostUpdate,
            (
                in_game::spawn_grounds,
                in_game::spawn_obstacles,
                in_game::check_for_collisions,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .run();
}
