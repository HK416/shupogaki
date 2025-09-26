// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::RestartResult),
            (debug_label, remove_resource, remove_entities),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label(mut next_state: ResMut<NextState<GameState>>) {
    info!("Current State: RestartResult");
    next_state.set(GameState::LoadInGame);
}

fn remove_resource(mut commands: Commands) {
    commands.remove_resource::<TrainFuel>();
    commands.remove_resource::<InputDelay>();
    commands.remove_resource::<CurrentScore>();
    commands.remove_resource::<CurrentState>();
    commands.remove_resource::<RetiredGrounds>();
    commands.remove_resource::<ObjectSpawner>();
}

fn remove_entities(
    mut commands: Commands,
    query_in_game_entities: Query<Entity, With<InGameStateRoot>>,
    query_result_entities: Query<Entity, With<ResultStateRoot>>,
) {
    for entity in query_in_game_entities.iter() {
        commands.entity(entity).despawn();
    }

    for entity in query_result_entities.iter() {
        commands.entity(entity).despawn();
    }
}
