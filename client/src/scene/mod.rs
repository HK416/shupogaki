mod assets;
mod constants;
mod resources;
mod system;
mod types;
mod utils;

mod in_game;
mod initialize;
mod option;
mod pause;
mod result;
mod setup;
mod title;

use bevy::prelude::*;

#[allow(unused_imports)]
pub use self::{assets::*, constants::*, resources::*, system::*, types::*, utils::*};

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(setup::StatePlugin)
            .add_plugins(initialize::StatePlugin)
            .add_plugins(option::StatePlugin)
            .add_plugins(pause::StatePlugin)
            .add_plugins(title::StatePlugin)
            .add_plugins(in_game::StatePlugin)
            .add_plugins(result::StatePlugin)
            .add_systems(Update, (initialize_font_size, update_font_size));
    }
}

// --- STATES ---

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    Error,
    Option,
    Pause,
    Resume,
    #[default]
    Setup,
    Initialize,
    LoadTitle,
    InitTitle,
    Title,
    Title2InGame,
    LoadInGame,
    InitInGame,
    ExitInGame,
    InitResult,
    PrepareInGame,
    StartInGame,
    InGame,
    WrapUpInGame,
    FinishedInGame,
    StartResult,
    Start2End,
    EndResult,
    CleanUpInGame,
    RestartResult,
    ExitResult,
}
