// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::model::ModelAsset;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::LoadInGame),
            (debug_label, load_in_game_assets, setup_loading_screen),
        )
        .add_systems(
            Update,
            (check_loading_progress, update_loading_bar).run_if(in_state(GameState::LoadInGame)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: LoadInGame");
}

fn load_in_game_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_assets = InGameAssets::default();

    // --- Sound Loading ---
    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_UI_START);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_UI_FINISH);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_TRAIN_START);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_1);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_2);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_TRAIN_END);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_TRAIN_LANDING);
    loading_assets.handles.push(sound.into());

    // --- Texture Loading ---
    let texture: Handle<Image> = asset_server.load(FONT_PATH_START);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_FINISH);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_NUMBER);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_TIME);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_SCORE);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_PAUSE);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_NUM_3);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_NUM_2);
    loading_assets.handles.push(texture.into());

    let texture: Handle<Image> = asset_server.load(FONT_PATH_NUM_1);
    loading_assets.handles.push(texture.into());

    let atlas: Handle<TextureAtlasLayout> = asset_server.load(ATLAS_PATH_NUMBER);
    loading_assets.handles.push(atlas.into());

    let texture: Handle<Image> = asset_server.load(TEXTURE_PATH_TRAIN_ICON);
    loading_assets.handles.push(texture.into());

    // --- Ground Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_PLANE_0);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_PLANE_999);
    loading_assets.handles.push(model.into());

    // --- Obstacle Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_BARRICADE);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_STONE);
    loading_assets.handles.push(model.into());

    // --- Item Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_FUEL);
    loading_assets.handles.push(model.into());

    // --- Player Loading ---
    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_00);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_01);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_TOY_TRAIN_02);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_HIKARI);
    loading_assets.handles.push(model.into());

    let model: Handle<ModelAsset> = asset_server.load(MODEL_PATH_NOZOMI);
    loading_assets.handles.push(model.into());

    // --- Animation Loading ---
    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_HIKARI_IN_GAME);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_HIKARI_VICTORY_START);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_HIKARI_VICTORY_END);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_NOZOMI_IN_GAME);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_NOZOMI_VICTORY_START);
    loading_assets.handles.push(clip.into());

    let clip: Handle<AnimationClip> = asset_server.load(ANIM_PATH_NOZOMI_VICTORY_END);
    loading_assets.handles.push(clip.into());

    // --- Resource Insertion ---
    commands.insert_resource(loading_assets);

    commands.remove_resource::<TitleAssets>();
}

fn setup_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, LoadingStateRoot));

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
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(88.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("Now Loading..."),
                        TextFont::from_font(font).with_font_size(24.0),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::WHITE,
                        ResizableFont::vertical(1280.0, 24.0),
                        Node::default(),
                        LoadingText,
                        ZIndex(2),
                    ));
                });

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
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BorderRadius::all(Val::Percent(50.0)),
                        BackgroundColor(LOADING_BAR_COLOR),
                        LoadingBar,
                        ZIndex(1),
                    ));
                });
        });

    commands.insert_resource(ClearColor(Color::BLACK));
}

// --- UPDATE SYSTEMS ---

fn check_loading_progress(
    asset_server: Res<AssetServer>,
    loading_assets: ResMut<InGameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let all_loaded = loading_assets
        .handles
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    if all_loaded {
        next_state.set(GameState::InitInGame);
    }
}

fn update_loading_bar(
    asset_server: Res<AssetServer>,
    loading_assets: Res<InGameAssets>,
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
