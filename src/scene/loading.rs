// Import necessary Bevy modules.
use bevy::{prelude::*, window::WindowResized};

use crate::asset::model::ModelAsset;

use super::*;

// --- COMPONENTS ---

/// A marker component for the loading bar UI entity.
#[derive(Component)]
pub struct LoadingBar;

/// A marker component for the "Now Loading..." text UI entity.
#[derive(Component)]
pub struct LoadingText;

// --- SETUP SYSTEM ---

/// Sets up the UI elements for the loading screen, including a 2D camera,
/// a progress bar, and "Now Loading..." text.
pub fn setup_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera for the loading screen UI.
    // This camera is marked with `LoadingStateEntity` to be cleaned up on exit.
    commands.spawn((Camera2d, LoadingStateEntity));

    // Create the "Now Loading..." text.
    let font = asset_server.load("fonts/NotoSans_Bold.ttf");
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
            LoadingStateEntity,
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

/// A system that loads all necessary game assets and tracks their loading progress.
pub fn load_necessary_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create resources to track loading progress and cache asset handles for later use.
    let mut loading_assets = LoadingAssets::default();

    // --- Ground Loading ---
    // Begin loading the primary ground model.
    let model: Handle<ModelAsset> = asset_server.load("models/Plane_0.hierarchy");
    // Add its handle to the `LoadingAssets` resource to track its loading status.
    loading_assets.handles.push(model.clone().into());

    // Load the ground model specifically for the result display area.
    let model: Handle<ModelAsset> = asset_server.load("models/Plane_999.hierarchy");
    // Add its handle to the `LoadingAssets` resource to track its loading status.
    loading_assets.handles.push(model.clone().into());

    // --- Obstacle Loading ---
    // Load the model for the fence obstacle.
    let model: Handle<ModelAsset> = asset_server.load("models/Barricade.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());

    // Load the model for the stone obstacle.
    let model: Handle<ModelAsset> = asset_server.load("models/Stone.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());

    // --- Item Loading ---
    // Load the model for the fuel item.
    let model: Handle<ModelAsset> = asset_server.load("models/Fuel.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());

    // --- Player and Toy Train Loading ---
    // Load all models and animations for the player and toy trains.
    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain00.hierarchy");
    loading_assets.handles.push(model.clone().into());
    // Load the second toy train model and attach the character to it.
    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain01.hierarchy");
    loading_assets.handles.push(model.clone().into());
    // Load the third toy train model and attach the character to it.
    let model: Handle<ModelAsset> = asset_server.load("models/ToyTrain02.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // Load player character (Hikari) animation and model.
    let clip: Handle<AnimationClip> = asset_server.load("animations/Hikari_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let clip: Handle<AnimationClip> =
        asset_server.load("animations/Hikari_Victory_Start_Interaction.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/Hikari.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // Load player character (Nozomi) animation and model.
    let clip: Handle<AnimationClip> = asset_server.load("animations/Nozomi_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let clip: Handle<AnimationClip> =
        asset_server.load("animations/Nozomi_Victory_Start_Interaction.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/Nozomi.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // --- Font Loading ---
    // Load the font for the text.
    let font: Handle<Font> = asset_server.load("fonts/NotoSans_Bold.ttf");
    // Add the font to the list of assets to track for loading.
    loading_assets.handles.push(font.clone().into());

    // --- Resource Insertion ---
    // Make the asset tracking and caching resources available to other systems.
    commands.insert_resource(loading_assets);
    // Set a simple black background for the loading screen.
    commands.insert_resource(ClearColor(Color::BLACK));
}

// --- CLEANUP SYSTEM ---

// A system that updates the loading text to "Boarding..." once all assets are loaded.
pub fn update_loading_text(mut query: Query<&mut Text, With<LoadingText>>) {
    if let Ok(mut text) = query.single_mut() {
        *text = Text::new("Boarding...")
    }
}

// --- UPDATE SYSTEM ---

/// A system that checks if all assets are loaded and transitions to the `Generate` state.
/// This runs continuously during the `GameState::Loading`.
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

    // If all assets are loaded, transition to the `Generate` state.
    if all_loaded {
        next_state.set(GameState::Generate);
    }
}

/// A system that adjusts the font size of the loading text when the window is resized
/// to maintain a consistent relative size.
pub fn change_text_scale(
    mut resize_reader: EventReader<WindowResized>,
    mut query: Query<&mut TextFont, With<LoadingText>>,
) {
    for e in resize_reader.read() {
        // Calculate the smaller of the window's width or height.
        let vmin = e.width.min(e.height);
        for mut text_font in query.iter_mut() {
            // Scale the font size based on the window's smaller dimension (vmin).
            // The font size of 18.0 is our baseline, designed for a 720px reference height.
            // This calculation maintains the font's relative size as the window resizes.
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

        // Calculate the loading progress as a value between 0.0 and 1.0.
        let total_count = loading_assets.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0 // Avoid division by zero if there are no assets to load.
        };

        // Update the width of the loading bar UI node to reflect the current progress.
        node.width = Val::Percent(progress * 100.0);
    }
}
