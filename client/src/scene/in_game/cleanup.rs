// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::CleanUpInGame),
            (
                debug_label,
                remove_entities,
                clear_player_effect,
                remove_effect_sounds,
                remove_voice_sounds,
            ),
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

fn remove_effect_sounds(mut commands: Commands, query: Query<Entity, With<EffectSound>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn remove_voice_sounds(mut commands: Commands, query: Query<Entity, With<VoiceSound>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
pub fn clear_player_effect(
    mut set: ParamSet<(
        Query<Entity, With<ToyTrain0>>,
        Query<Entity, With<ToyTrain1>>,
        Query<Entity, With<ToyTrain2>>,
    )>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(entity) = set.p0().single() {
        clear_player_effect_recursive(entity, &children_query, &material_query, &mut materials);
    }

    if let Ok(entity) = set.p1().single() {
        clear_player_effect_recursive(entity, &children_query, &material_query, &mut materials);
    }

    if let Ok(entity) = set.p2().single() {
        clear_player_effect_recursive(entity, &children_query, &material_query, &mut materials);
    }
}

fn clear_player_effect_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(handle) = material_query.get(entity)
        && let Some(material) = materials.get_mut(handle.id())
    {
        material.base_color = Color::WHITE;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            clear_player_effect_recursive(child, children_query, material_query, materials);
        }
    }
}
