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
    // Initialize resources to store asset handles and cached models.
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

    // --- Obstacle Loading ---
    // Load the rail obstacle model.
    let model: Handle<ModelAsset> = asset_server.load("models/Rail_0.hierarchy");
    // Add its handle to the loading queue.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle for later use in spawning obstacles during the game.
    cached_obstacles
        .models
        .insert(ObstacleModel::Rail0, model.clone());

    // --- Player and Toy Train Loading ---
    // Load all models and animations for the player and toy trains.
    // They are spawned with `Visibility::Hidden` and will be made visible after loading is complete.

    // Load the first toy train model.
    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain00.hierarchy");
    loading_assets.handles.push(model.clone().into());
    commands.spawn((
        SpawnModel(model),
        Transform::default(),
        InGameStateEntity,
        Visibility::Hidden,
        ToyTrain0,
    ));

    // Load player character (Hikari) animation and model.
    let clip: Handle<AnimationClip> = asset_server.load("animations/CH0242_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/Hikari.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // Spawn the character entity, which will be a child of the toy train
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.8775, 0.0),
            InGameStateEntity,
            Visibility::Hidden,
        ))
        .id();

    // Load the second toy train model and attach the character to it.
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
        .add_child(entity); // Hikari is a child of this train.

    // Load player character (Nozomi) animation and model.
    let clip: Handle<AnimationClip> = asset_server.load("animations/CH0243_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/Nozomi.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // Spawn the character entity, which will be a child of the toy train
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.375, 0.375),
            InGameStateEntity,
            Visibility::Hidden,
        ))
        .id();

    // Load the third toy train model and attach the character to it.
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
        .add_child(entity); // Nozomi is a child of this train.

    // Create the loading UI.
    create_loading_ui(&mut commands, &asset_server, &mut loading_assets);
    create_in_game_ui(&mut commands, &asset_server, &mut loading_assets);

    // --- Resource Insertion ---
    // Insert the `loading_assets` and `cached_grounds` resources for other systems to use.
    commands.insert_resource(loading_assets);
    commands.insert_resource(cached_grounds);
    commands.insert_resource(cached_obstacles);
    // Set a black background color for the loading screen.
    commands.insert_resource(ClearColor(Color::BLACK));
}

/// Creates the in-game UI elements, specifically the score display.
/// The score is composed of multiple digit images, which are pre-loaded here.
/// The UI is initially hidden and will be made visible when the game starts.
fn create_in_game_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    loading_assets: &mut LoadingAssets,
) {
    // Load the sprite sheet and texture atlas for the number font.
    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Number.sprite");
    loading_assets.handles.push(texture_handle.clone().into());

    let texture_atlas_handle: Handle<TextureAtlasLayout> =
        asset_server.load("fonts/ImgFont_Number.atlas");
    loading_assets
        .handles
        .push(texture_atlas_handle.clone().into());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(1.5),
                left: Val::Vw(1.5),
                width: Val::Vw(25.0),
                height: Val::Vw(5.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            Score,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace100000s, // Marker for the 100,000s place digit.
            ));

            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace10000s, // Marker for the 10,000s place digit.
            ));

            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace1000s, // Marker for the 1,000s place digit.
            ));

            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace100s, // Marker for the 100s place digit.
            ));

            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace10s, // Marker for the 10s place digit.
            ));

            parent.spawn((
                ImageNode::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas::from(texture_atlas_handle.clone()),
                ),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace1s, // Marker for the 1s place digit.
            ));
        });

    let texture_handle: Handle<Image> = asset_server.load("textures/Train_Icon.sprite");
    loading_assets.handles.push(texture_handle.clone().into());

    // Create the root node for the fuel gauge in the bottom-right corner.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Vh(1.5),
                right: Val::Vw(3.0),
                width: Val::Vw(25.0),
                height: Val::Vw(5.0),
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            ViewVisibility::HIDDEN, // Use ViewVisibility to control the entire hierarchy's visibility.
        ))
        .with_children(|parent| {
            // Create the background/border of the fuel gauge.
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(15.0),
                        bottom: Val::Px(0.0),
                        border: UiRect::all(Val::Percent(1.0)),
                        ..Default::default()
                    },
                    BackgroundColor(FUEL_COLOR),
                    BorderColor(FUEL_COLOR),
                    BorderRadius::all(Val::Percent(50.0)),
                    Visibility::Inherited,
                    ZIndex(2), // Ensure the border is drawn above the gauge bar.
                ))
                .with_children(|parent| {
                    // Create the actual fuel gauge bar that will change width.
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0), // Starts full
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BackgroundColor(FUEL_GAUGE_COLOR),
                        BorderRadius::all(Val::Percent(50.0)),
                        Visibility::Inherited,
                        ZIndex(1), // Drawn below the border.
                        FuelGauge, // Marker to identify this entity for updates.
                    ));
                });

            // Create the decorative train icon next to the fuel gauge.
            parent.spawn((
                ImageNode::new(texture_handle.clone()).with_color(FUEL_COLOR),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Percent(80.0),
                    bottom: Val::Percent(12.5),
                    left: Val::Px(0.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ZIndex(2), // Ensure it's drawn on top.
                FuelDeco,  // Marker component.
            ));
        });
}

/// Sets up the UI elements for the loading screen, including a 2D camera,
/// a progress bar, and "Now Loading..." text.
fn create_loading_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    loading_assets: &mut LoadingAssets,
) {
    // Spawn a 2D camera for the loading screen UI.
    // This camera is marked with `InGameLoadStateEntity` to be cleaned up on exit.
    commands.spawn((Camera2d, InGameLoadStateEntity));

    // Create the "Now Loading..." text.
    // Load the font for the text.
    let font: Handle<Font> = asset_server.load("fonts/NotoSans_Bold.ttf");
    // Add the font to the list of assets to track for loading.
    loading_assets.handles.push(font.clone().into());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::VMin(20.0),
                height: Val::VMin(1.0),
                bottom: Val::Vh(3.0),
                right: Val::Vw(3.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameLoadStateEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Now Loading..."),
                TextFont::from_font(font).with_font_size(18.0),
                TextColor::WHITE,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(150.0),
                    ..Default::default()
                },
                ZIndex(2),
                LoadingText, // Marker component to find and update this text.
            ));

            // Create the container for the loading bar.
            // `Overflow::hidden` clips the inner bar as it grows.
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(80.0),
                        overflow: Overflow::hidden(),
                        ..Default::default()
                    },
                    ZIndex(1),
                ))
                .with_children(|parent| {
                    // Spawn the loading bar itself, which will grow in width from 0% to 100%.
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0), // Starts at 0% width.
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BorderRadius::all(Val::Percent(50.0)),
                        BackgroundColor(LOADING_BAR_COLOR),
                        LoadingBar, // Marker component to find and update this loading bar.
                    ));
                });
        });
}

// --- CLEANUP SYSTEM ---

/// A system that cleans up the loading screen and makes the game objects visible.
/// This runs once when transitioning from `InGameLoading` to the `InGame` state.
pub fn on_exit(
    mut commands: Commands,
    // Query for all entities that are part of the loading screen.
    load_entities: Query<Entity, With<InGameLoadStateEntity>>,
    // Query for all entities that are part of the main game.
    mut in_game_entities: Query<&mut Visibility, With<InGameStateEntity>>,
) {
    // Remove resources that were specific to the loading state.
    commands.remove_resource::<ClearColor>(); // Revert to the default background color.
    commands.remove_resource::<LoadingAssets>(); // No longer need to track loading assets.

    // Despawn all entities associated with the loading screen (UI camera, text, loading bar).
    for entity in load_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Make all the pre-spawned game objects (player, ground, etc.) visible.
    for mut visibility in in_game_entities.iter_mut() {
        *visibility = Visibility::Visible;
    }

    info!("Loading complete! Starting game...");
}

// --- UPDATE SYSTEM ---

/// A system that checks if all assets are loaded and transitions to the `InGame` state.
/// This runs continuously during the `InGameLoading` state.
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

/// A system that adjusts the font size of the loading text when the window is resized
/// to maintain a consistent relative size.
pub fn change_text_scale(
    mut resize_reader: EventReader<WindowResized>,
    mut query: Query<&mut TextFont, With<LoadingText>>,
) {
    for e in resize_reader.read() {
        let vmin = e.width.min(e.height);
        for mut text_font in query.iter_mut() {
            text_font.font_size = 18.0 * vmin / 720.0;
            info!("Resize Font size:{:?}", text_font.font_size);
        }
    }
}

/// A system that updates the loading bar's width based on the number of loaded assets.
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
