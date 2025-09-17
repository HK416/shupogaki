mod cleanup;
mod end;
mod restart;
mod start;
mod start_to_end;

// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- CONSTANTS ---
pub const HIKARI_POSITION: Vec3 = vec3(-5.0, 0.0, -0.383545);
pub const NOZOMI_POSITION: Vec3 = vec3(-5.0, 0.0, 0.383545);
pub const STUDENT_DIRECTION: Vec3 = vec3(1.0, 0.0, 0.0);
pub const CAMERA_POSITION: Vec3 = vec3(-2.0, 1.0, 0.0);
pub const CAMERA_DIRECTION: Vec3 = vec3(-0.995037, -0.0995037, 0.0);

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(start::StatePlugin)
            .add_plugins(start_to_end::StatePlugin)
            .add_plugins(end::StatePlugin)
            .add_plugins(restart::StatePlugin)
            .add_plugins(cleanup::StatePlugin);
    }
}
