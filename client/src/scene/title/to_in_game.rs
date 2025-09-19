// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Title2InGame),
            (
                debug_label,
                enter_next_state,
                remove_title_entities,
                remove_leader_board_entities,
            ),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Title2InGame");
}

fn enter_next_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::LoadInGame);
}

fn remove_title_entities(mut commands: Commands, query: Query<Entity, With<TitleStateRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn remove_leader_board_entities(
    mut commands: Commands,
    query: Query<Entity, With<RankingStateRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
