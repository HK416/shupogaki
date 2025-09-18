// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::model::ModelAsset;

use super::*;

// --- CONSTANTS ---

const TIMEOUT: f32 = 5.0;
const MAX_RETRY_COUNT: u32 = 5;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::LoadTitle),
            (
                debug_label,
                load_title_assets,
                setup_loading_screen,
                init_asset_load_timeout_retry,
            ),
        )
        .add_systems(
            OnExit(GameState::LoadTitle),
            cleanup_asset_load_timeout_retry,
        )
        .add_systems(
            Update,
            (
                check_loading_progress,
                update_loading_progress,
                check_and_retry_asset_load_timeout,
            )
                .run_if(in_state(GameState::LoadTitle)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: LoadTitle");
}

fn load_title_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_assets(&mut commands, &asset_server);
}

fn load_assets(commands: &mut Commands, asset_server: &AssetServer) {
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

    // --- Sound Loading ----
    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_HIKARI_TITLE);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_NOZOMI_TITLE);
    loading_assets.handles.push(sound.into());

    let sound: Handle<AudioSource> = asset_server.load(SOUND_PATH_SFX_DOOR_BELL_00);
    loading_assets.handles.push(sound.into());

    commands.insert_resource(loading_assets);
}

fn setup_loading_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<(), With<Camera3d>>,
) {
    if camera_query.single().is_err() {
        // Spawn a 3D camera for the loading screen UI.
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, -100.0, 0.0).looking_to(Vec3::NEG_Y, Vec3::Z),
            LoadingStateRoot,
        ));

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

fn init_asset_load_timeout_retry(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
    commands.insert_resource(Counter::default());
}

// --- CLEANUP SYSTEMS ---

fn cleanup_asset_load_timeout_retry(mut commands: Commands) {
    commands.remove_resource::<SceneTimer>();
    commands.remove_resource::<Counter>();
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

fn update_loading_progress(
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

fn check_and_retry_asset_load_timeout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut counter: ResMut<Counter>,
    mut scene_timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    scene_timer.tick(time.delta_secs());
    if scene_timer.elapsed_time >= TIMEOUT {
        scene_timer.reset();

        **counter += 1;
        if **counter > MAX_RETRY_COUNT {
            error!("Asset load request timed out.");
            next_state.set(GameState::Error);
        } else {
            load_assets(&mut commands, &asset_server);
        }
    }
}
