// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::material::EyeMouthMaterial;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::ExitInGame),
            (
                debug_label,
                remove_resource,
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
    info!("Current State: ExitInGame");
    next_state.set(GameState::LoadTitle);
}

fn remove_resource(mut commands: Commands) {
    commands.remove_resource::<Attacked>();
    commands.remove_resource::<PlayTime>();
    commands.remove_resource::<TrainFuel>();
    commands.remove_resource::<InputDelay>();
    commands.remove_resource::<CurrentLane>();
    commands.remove_resource::<CurrentScore>();
    commands.remove_resource::<ForwardMovement>();
    commands.remove_resource::<VerticalMovement>();
    commands.remove_resource::<IsPlayerJumping>();
    commands.remove_resource::<CurrentState>();
    commands.remove_resource::<RetiredGrounds>();
    commands.remove_resource::<ObjectSpawner>();
    commands.remove_resource::<InGameAssets>();
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
fn clear_player_effect(
    mut set: ParamSet<(
        Query<Entity, With<ToyTrain0>>,
        Query<Entity, With<ToyTrain1>>,
        Query<Entity, With<ToyTrain2>>,
    )>,
    children_query: Query<&Children>,
    base_color_query: Query<&BaseColor>,
    standard_material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    extented_material_query: Query<&MeshMaterial3d<EyeMouthMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut extended_materials: ResMut<Assets<EyeMouthMaterial>>,
) {
    if let Ok(entity) = set.p0().single() {
        clear_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
        );
    }

    if let Ok(entity) = set.p1().single() {
        clear_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
        );
    }

    if let Ok(entity) = set.p2().single() {
        clear_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
        );
    }
}

fn clear_player_effect_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    base_color_query: &Query<&BaseColor>,
    standard_material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    extented_material_query: &Query<&MeshMaterial3d<EyeMouthMaterial>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    extended_materials: &mut ResMut<Assets<EyeMouthMaterial>>,
) {
    if let Ok(handle) = standard_material_query.get(entity)
        && let Some(material) = standard_materials.get_mut(handle.id())
    {
        material.base_color = base_color_query
            .get(entity)
            .map(|c| c.0)
            .unwrap_or(Color::WHITE);
    }

    if let Ok(handle) = extented_material_query.get(entity)
        && let Some(material) = extended_materials.get_mut(handle.id())
    {
        material.base.base_color = base_color_query
            .get(entity)
            .map(|c| c.0)
            .unwrap_or(Color::WHITE);
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            clear_player_effect_recursive(
                child,
                children_query,
                base_color_query,
                standard_material_query,
                extented_material_query,
                standard_materials,
                extended_materials,
            );
        }
    }
}
