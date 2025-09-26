mod init;
mod load;
mod main;
mod to_in_game;

// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- CONSTANTS ---
pub const TRAIN_POSITION: Vec3 = vec3(LANE_POSITIONS[0], 0.0, 0.0);
pub const HIKARI_POSITION: Vec3 = vec3(-2.8438, 0.0, 2.8438);
pub const NOZOMI_POSITION: Vec3 = vec3(-2.0, 0.0, 2.0);
pub const CAMERA_POSITION: Vec3 = vec3(-1.0, 1.0, 4.5);
pub const CAMERA_DIRECTION: Vec3 = vec3(-0.703163, -0.105474, -0.703163);

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(load::StatePlugin)
            .add_plugins(init::StatePlugin)
            .add_plugins(main::StatePlugin)
            .add_plugins(to_in_game::StatePlugin);
    }
}
