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
            (debug_label, load_title_assets, setup_loading_screen),
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

fn setup_loading_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<(), With<Camera2d>>,
) {
    if camera_query.single().is_err() {
        // Spawn a 2D camera for the loading screen UI.
        commands.spawn((Camera2d, LoadingStateRoot));

        // Create the main UI container for the loading elements.
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Vw(20.0),
                    height: Val::Vh(5.0),
                    bottom: Val::Vh(3.0),
                    right: Val::Vw(3.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                LoadingStateRoot,
            ))
            .with_children(|parent| {
                // Container for the loading text.
                parent
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(88.0),
                        ..Default::default()
                    },))
                    .with_children(|parent| {
                        // Spawn the "Now Loading..." text element.
                        let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                        parent.spawn((
                            Text::new("Now Loading..."),
                            TextFont::from_font(font).with_font_size(24.0),
                            TextLayout::new_with_justify(JustifyText::Center),
                            TextColor::WHITE,
                            ResizableFont::vertical(1280.0, 24.0),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                overflow: Overflow::hidden(),
                                ..Default::default()
                            },
                            LoadingText,
                            ZIndex(2),
                        ));
                    });

                // Container for the loading progress bar.
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(12.0),
                            border: UiRect::all(Val::Percent(0.25)),
                            ..Default::default()
                        },
                        BorderColor(Color::WHITE),
                        BorderRadius::all(Val::Percent(50.0)),
                    ))
                    .with_children(|parent| {
                        // The actual loading bar that will be filled.
                        parent.spawn((
                            Node {
                                width: Val::Percent(0.0), // Starts at 0% width.
                                height: Val::Percent(100.0),
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(50.0)),
                            BackgroundColor(LOADING_BAR_COLOR),
                            LoadingBar, // Tag component for querying.
                            ZIndex(1),
                        ));
                    });
            });

        // Set the background color to black during setup.
        commands.insert_resource(ClearColor(Color::BLACK));
    }
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
