// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::model::ModelAsset;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::LoadTitle),
            (debug_label, load_title_assets),
        )
        .add_systems(
            Update,
            (check_loading_progress, update_loading_bar).run_if(in_state(GameState::LoadTitle)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: LoadTitle");
}

fn load_title_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_assets = TitleAssets::default();

    // --- Ground Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_PLANE_0);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_PLANE_999);
    loading_assets.handles.push(model.into());

    // --- Toy Train Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_00);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_01);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_02);
    loading_assets.handles.push(model.into());

    // --- Student Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_HIKARI);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_NOZOMI);
    loading_assets.handles.push(model.into());

    // --- Animation Loading ---
    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_HIKARI_CAFE_IDLE);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_NOZOMI_CAFE_IDLE);
    loading_assets.handles.push(clip.into());

    commands.insert_resource(loading_assets);
}

// --- UPDATE SYSTEMS ---

fn check_loading_progress(
    asset_server: Res<AssetServer>,
    loading_assets: ResMut<TitleAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let all_loaded = loading_assets
        .handles
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    if all_loaded {
        next_state.set(GameState::InitTitle);
    }
}

fn update_loading_bar(
    asset_server: Res<AssetServer>,
    loading_assets: Res<TitleAssets>,
    mut query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = query.single_mut() {
        let loaded_count = loading_assets
            .handles
            .iter()
            .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
            .count();

        let total_count = loading_assets.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0
        };

        node.width = Val::Percent(progress * 100.0);
    }
}
