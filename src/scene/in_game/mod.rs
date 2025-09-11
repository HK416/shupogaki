mod cleanup;
mod exit;
mod finished;
mod init_in_game;
mod init_result;
mod load;
mod main;
mod prepare;
mod resume;
mod start;
mod wrapup;

// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(load::StatePlugin)
            .add_plugins(init_in_game::StatePlugin)
            .add_plugins(init_result::StatePlugin)
            .add_plugins(prepare::StatePlugin)
            .add_plugins(resume::StatePlugin)
            .add_plugins(start::StatePlugin)
            .add_plugins(main::StatePlugin)
            .add_plugins(wrapup::StatePlugin)
            .add_plugins(finished::StatePlugin)
            .add_plugins(cleanup::StatePlugin)
            .add_plugins(exit::StatePlugin);
    }
}
