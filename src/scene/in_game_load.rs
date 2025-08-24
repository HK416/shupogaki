// Import necessary Bevy modules.
use bevy::{prelude::*, window::WindowResized};

use crate::asset::{model::ModelAsset, spawner::SpawnModel};

use super::*;

// --- COMPONENTS ---

/// A marker component for the loading bar UI entity.
#[derive(Component)]
pub struct LoadingBar;

/// A marker component for the "Now Loading..." text UI entity.
#[derive(Component)]
pub struct LoadingText;

// --- RESOURCES ---

/// A resource to store handles of assets that need to be loaded before the game can start.
#[derive(Default, Resource)]
pub struct LoadingAssets {
    handles: Vec<UntypedHandle>,
}

// --- SETUP SYSTEM ---

/// A system that runs once when entering the `InGameLoading` state.
/// It sets up the loading screen UI and starts loading all necessary game assets.
pub fn on_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_assets = LoadingAssets::default();
    let mut cached_grounds = CachedGrounds::default();
    let mut cached_obstacles = CachedObstacles::default();

    // --- Ground Loading and Pre-spawning ---
    // Load the ground model asset.
    let model: Handle<ModelAsset> = asset_server.load("models/Plane_0.hierarchy");
    // Add the ground model handle to the list of assets to be loaded.
    loading_assets.handles.push(model.clone().into());
    // Cache the ground model handle for later use in the game.
    cached_grounds
        .models
        .insert(GroundModel::Plane0, model.clone());
    // Pre-spawn initial ground entities. These are initially hidden and will be made
    // visible when the game starts. This avoids a stutter when the game begins.
    for i in 0..5 {
        commands.spawn((
            SpawnModel(model.clone()),
            Transform::from_xyz(0.0, 0.0, 30.0 * i as f32),
            InGameStateEntity,
            Visibility::Hidden,
            Ground,
        ));
    }

    let model: Handle<ModelAsset> = asset_server.load("models/Rail_0.hierarchy");
    loading_assets.handles.push(model.clone().into());
    cached_obstacles
        .models
        .insert(ObstacleModel::Rail0, model.clone());

    // --- Player and Toy Train Loading ---
    // Load all models and animations for the player and toy trains.
    // They are spawned with `Visibility::Hidden` and will be made visible after loading is complete.
    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain00.hierarchy");
    loading_assets.handles.push(model.clone().into());
    commands.spawn((
        SpawnModel(model),
        Transform::default(),
        InGameStateEntity,
        Visibility::Hidden,
        ToyTrain0,
    ));

    let clip: Handle<AnimationClip> = asset_server.load("animations/CH0242_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/CH0242.hierarchy");
    loading_assets.handles.push(model.clone().into());
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.8775, 0.0),
            InGameStateEntity,
            Visibility::Hidden,
        ))
        .id();

    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain01.hierarchy");
    loading_assets.handles.push(model.clone().into());
    commands
        .spawn((
            SpawnModel(model),
            Transform::default(),
            InGameStateEntity,
            Visibility::Hidden,
            ToyTrain1,
        ))
        .add_child(entity);

    let clip: Handle<AnimationClip> = asset_server.load("animations/CH0243_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/CH0243.hierarchy");
    loading_assets.handles.push(model.clone().into());
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.375, 0.375),
            InGameStateEntity,
            Visibility::Hidden,
        ))
        .id();

    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain02.hierarchy");
    loading_assets.handles.push(model.clone().into());
    commands
        .spawn((
            SpawnModel(model),
            Transform::default(),
            InGameStateEntity,
            Visibility::Hidden,
            ToyTrain2,
        ))
        .add_child(entity);

    // Create the loading UI.
    create_loading_ui(&mut commands, &asset_server, &mut loading_assets);

    // --- Resource Insertion ---
    // Insert the `loading_assets` and `cached_grounds` resources for other systems to use.
    commands.insert_resource(loading_assets);
    commands.insert_resource(cached_grounds);
    commands.insert_resource(cached_obstacles);
    // Set a black background color for the loading screen.
    commands.insert_resource(ClearColor(Color::BLACK));
}

/// A function that creates the loading UI.
fn create_loading_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    loading_assets: &mut LoadingAssets,
) {
    // UI camera.
    commands.spawn((Camera2d, InGameLoadStateEntity));

    // Loading Bar.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(18.0),
                height: Val::Vh(5.0),
                bottom: Val::Vh(3.0),
                right: Val::Vw(3.0),
                border: UiRect::all(Val::Percent(0.3)),
                ..Default::default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderColor(Color::srgb(0.85, 0.85, 0.85)),
            InGameLoadStateEntity,
        ))
        .with_children(|parent| {
            // Prograss Bar.
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                LoadingBar,
            ));
        });

    // Loading text.
    let font: Handle<Font> = asset_server.load("fonts/NotoSans_Bold.ttf");
    loading_assets.handles.push(font.clone().into());
    commands.spawn((
        Text::new("Now Loading..."),
        TextFont {
            font,
            font_size: 18.0,
            ..Default::default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Vw(18.0),
            height: Val::Vh(4.0),
            bottom: Val::Vh(8.0),
            right: Val::Vw(3.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        InGameLoadStateEntity,
        LoadingText,
    ));
}

// --- CLEANUP SYSTEM ---

/// A system that cleans up the loading screen and makes the game objects visible.
pub fn on_exit(
    mut commands: Commands,
    load_entities: Query<Entity, With<InGameLoadStateEntity>>,
    mut in_game_entities: Query<&mut Visibility, With<InGameStateEntity>>,
) {
    // Remove resources.
    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<LoadingAssets>();

    // Despawn all entities associated with the loading screen.
    for entity in load_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Make all game objects visible.
    for mut visibility in in_game_entities.iter_mut() {
        *visibility = Visibility::Visible;
    }

    info!("Loading complete! Starting game...");
}

// --- UPDATE SYSTEM ---

/// A system that checks if all assets are loaded and transitions to the `InGame` state.
pub fn check_loading_progress(
    asset_server: Res<AssetServer>,
    loading_assets: Res<LoadingAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check if all assets are loaded.
    let all_loaded = loading_assets
        .handles
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    // If all assets are loaded, transition to the `InGame` state.
    if all_loaded {
        next_state.set(GameState::InGame);
    }
}

/// A system that adjusts the font size of the loading text when the window is resized.
pub fn change_text_scale(
    mut resize_reader: EventReader<WindowResized>,
    mut query: Query<&mut TextFont, With<LoadingText>>,
) {
    for e in resize_reader.read() {
        let vmin = e.width.min(e.height);
        for mut text_font in query.iter_mut() {
            text_font.font_size = 18.0 * vmin / 720.0;
            info!("{:?}", text_font.font_size);
        }
    }
}

/// A system that updates the loading bar based on the number of loaded assets.
pub fn update_loading_bar(
    asset_server: Res<AssetServer>,
    loading_assets: Res<LoadingAssets>,
    mut loading_bar_query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = loading_bar_query.single_mut() {
        // Count the number of loaded assets.
        let loaded_count = loading_assets
            .handles
            .iter()
            .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
            .count();

        // Calculate the progress.
        let total_count = loading_assets.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0
        };

        // Update the width of the loading bar.
        node.width = Val::Percent(progress * 100.0);
    }
}
