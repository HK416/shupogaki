// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Title2InGame),
            (debug_label, remove_entities),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Title2InGame");
}

fn remove_entities(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<TitleStateRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    next_state.set(GameState::LoadInGame);
}
