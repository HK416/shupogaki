// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::CleanUpInGame),
            (debug_label, remove_entities),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label(mut next_state: ResMut<NextState<GameState>>) {
    info!("Current State: CleanUpInGame");
    next_state.set(GameState::StartResult);
}

fn remove_entities(mut commands: Commands, query: Query<Entity, With<InGameStateRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
