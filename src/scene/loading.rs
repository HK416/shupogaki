// src/scene/loading.rs

//! This module handles the `Loading` game state. It is responsible for:
//! - Displaying a loading screen with a progress bar.
//! - Loading all necessary game assets, including models, animations, and UI textures.
//! - Pre-spawning initial game entities (like the ground and player) and hiding them
//!   to prevent stuttering when the game starts.
//! - Transitioning to the `Prepare` state once all assets are loaded.

use std::time::Duration;

// Import necessary Bevy modules.
use bevy::{prelude::*, window::WindowResized};
use bevy_tweening::{Animator, AnimatorState, Tween, lens::UiPositionLens};

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

/// A system that runs once when entering the `GameState::Loading`.
///
/// This function is responsible for the bulk of the initial setup. It orchestrates:
/// 1.  **Asset Loading**: It begins loading all models, animations, and textures required for the game.
/// 2.  **Resource Creation**: It creates and inserts the resources that will track loading progress (`LoadingAssets`)
///     and cache asset handles for later use (`CachedGrounds`, `CachedObjects`).
/// 3.  **Entity Pre-spawning**: It pre-spawns the initial game entities (player, ground, obstacles) with
///     `Visibility::Hidden`. This "pre-warming" technique prevents performance stutters that might
///     occur if these entities were all spawned at once when gameplay begins.
/// 4.  **UI Creation**: It calls helper functions to build both the loading screen UI and the in-game UI,
///     which is also initially hidden.
pub fn on_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create resources to track loading progress and cache asset handles for later use.
    let mut loading_assets = LoadingAssets::default();
    let mut cached_grounds = CachedGrounds::default();
    let mut cached_objects = CachedObjects::default();

    // --- Ground Loading and Pre-spawning ---
    // Begin loading the primary ground model.
    let model: Handle<ModelAsset> = asset_server.load("models/Plane_0.hierarchy");
    // Add its handle to the `LoadingAssets` resource to track its loading status.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle in the `CachedGrounds` resource for efficient reuse during gameplay.
    cached_grounds
        .models
        .insert(GroundModel::Plane0, model.clone());
    // Pre-spawn the initial ground segments. They are created with `Visibility::Hidden` and will be made
    // visible when the game starts. This technique, often called "object pooling" or "pre-warming",
    // helps prevent a performance stutter that could occur if these entities were all spawned at once at the beginning of the game.
    for i in 0..7 {
        commands.spawn((
            SpawnModel(model.clone()),
            Transform::from_xyz(0.0, 0.0, DESPAWN_LOCATION + 30.0 * i as f32),
            InGameStateEntity,
            Visibility::Hidden,
            Ground,
        ));
    }

    // Load the ground model specifically for the result display area.
    let model: Handle<ModelAsset> = asset_server.load("models/Plane_999.hierarchy");
    // Add its handle to the `LoadingAssets` resource to track its loading status.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle in the `CachedGrounds` resource, associating it with `GroundModel::Plane999`.
    cached_grounds
        .models
        .insert(GroundModel::Plane999, model.clone());

    // --- Obstacle Loading ---
    // Load the model for the fence obstacle.
    let model: Handle<ModelAsset> = asset_server.load("models/Barricade.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle for reuse when spawning new obstacles during gameplay.
    cached_objects.models.insert(SpawnObject::Fence0, model);

    // Load the model for the stone obstacle.
    let model: Handle<ModelAsset> = asset_server.load("models/Stone_0.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle for reuse when spawning new obstacles during gameplay.
    cached_objects.models.insert(SpawnObject::Stone0, model);

    // --- Item Loading ---
    // Load the model for the fuel item.
    let model: Handle<ModelAsset> = asset_server.load("models/Fuel.hierarchy");
    // Add its handle to the loading tracker.
    loading_assets.handles.push(model.clone().into());
    // Cache the handle for reuse when spawning new fuel items during gameplay.
    cached_objects.models.insert(SpawnObject::Fuel, model);

    // --- Player and Toy Train Loading ---
    // Load all models and animations for the player and toy trains.
    // They are spawned with `Visibility::Hidden` and will be made visible after loading is complete.
    // This follows the same pre-spawning pattern as the ground to ensure a smooth game start.
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

    // Spawn the character entity. It will be parented to one of the toy train cars.
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.8775, 0.0),
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
        .add_child(entity); // Parent the Hikari model to this train car.

    // Load player character (Nozomi) animation and model.
    let clip: Handle<AnimationClip> = asset_server.load("animations/CH0243_InGame.anim");
    loading_assets.handles.push(clip.clone().into());
    let model: Handle<ModelAsset> = asset_server.load("models/Nozomi.hierarchy");
    loading_assets.handles.push(model.clone().into());

    // Spawn the character entity, which will be parented to another toy train car.
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_xyz(0.0, 0.5, 0.375),
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
        .add_child(entity); // Parent the Nozomi model to this train car.

    // --- UI Loading ---
    // Set up the UI for both the loading screen and the main game.
    // These are also loaded upfront to prevent hitches.
    create_loading_ui(&mut commands, &asset_server, &mut loading_assets);
    create_in_game_ui(&mut commands, &asset_server, &mut loading_assets);
    create_pause_ui(&mut commands, &asset_server, &mut loading_assets);

    // --- Resource Insertion ---
    // Make the asset tracking and caching resources available to other systems.
    commands.insert_resource(loading_assets);
    commands.insert_resource(cached_grounds);
    commands.insert_resource(cached_objects);
    // Set a simple black background for the loading screen.
    commands.insert_resource(ClearColor(Color::BLACK));
}

/// Creates the in-game UI elements.
///
/// The UI is built here during the `Loading` state to ensure all its assets (fonts, textures)
/// are loaded upfront. The entire UI is spawned with `Visibility::Hidden` and its entrance
/// animations are set to `AnimatorState::Paused`. The animations will be started manually
/// when the `InGame` state is entered, creating a smooth slide-in effect.
fn create_in_game_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    loading_assets: &mut LoadingAssets,
) {
    // Load the texture for the "Start" message.
    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Start.sprite");
    // Add the handle to the list of assets to track for loading progress.
    loading_assets.handles.push(texture_handle.clone().into());

    commands
        // Spawn the root node for the "Start" UI element.
        // This node acts as a container for the "Start" image.
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            UI::Start,
        ))
        .with_children(|parent| {
            // Spawn the image node for the "Start" message.
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                StartAnimation::new(UI_ANIMATION_DURATION),
                Visibility::Inherited,
                ZIndex(4),
            ));
        });

    // Load the texture for the "Finish" message.
    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Finish.sprite");
    // Add the handle to the list of assets to track for loading progress.
    loading_assets.handles.push(texture_handle.clone().into());

    // Spawn the root node for the "Finish" UI element.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            UI::Finish,
        ))
        .with_children(|parent| {
            // Spawn the image node for the "Finish" message.
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                FinishAnimation::new(UI_ANIMATION_DURATION),
                Visibility::Inherited,
                ZIndex(4),
            ));
        });

    // Spawn the pause button UI element.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(1.5),
                right: Val::Vw(1.5),
                width: Val::Vw(4.5),
                height: Val::Vw(4.5),
                ..Default::default()
            },
            BorderRadius::all(Val::Percent(30.0)),
            BackgroundColor(PAUSE_BTN_COLOR),
            // Defines a drop shadow for the button to give it some depth.
            // Parameters: color, h_offset, v_offset, spread, blur_radius
            BoxShadow::new(
                Color::BLACK.with_alpha(0.3),
                Val::Percent(0.0),
                Val::Percent(0.0),
                Val::Percent(0.0),
                Val::Px(10.0),
            ),
            Animator::new(Tween::new(
                // Define the animation for the pause button sliding in from the top.
                EaseFunction::SmoothStep,
                Duration::from_secs_f32(UI_ANIMATION_DURATION),
                UiPositionLens {
                    start: UiRect {
                        left: Val::Auto,
                        right: Val::Vw(1.5),
                        top: Val::Vh(-20.0),
                        bottom: Val::Auto,
                    },
                    end: UiRect {
                        left: Val::Auto,
                        right: Val::Vw(1.5),
                        top: Val::Vh(1.5),
                        bottom: Val::Auto,
                    },
                },
            ))
            .with_state(AnimatorState::Paused),
            InGameStateEntity,
            Visibility::Hidden,
            UI::PauseButton, // Marker component for the pause button.
            ZIndex(1),
            Button,
        ))
        // Add children to create the pause icon (two vertical bars).
        .with_children(|parent| {
            // The left vertical bar of the pause icon.
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(20.0),
                    left: Val::Percent(30.0),
                    width: Val::Percent(15.0),
                    height: Val::Percent(60.0),
                    ..Default::default()
                },
                BorderRadius::all(Val::Percent(50.0)),
                BackgroundColor(PAUSE_ICON_COLOR),
                // Inherit visibility from parent, so it's hidden initially.
                Visibility::Inherited,
                ZIndex(2),
            ));

            // The right vertical bar of the pause icon.
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(20.0),
                    right: Val::Percent(30.0),
                    width: Val::Percent(15.0),
                    height: Val::Percent(60.0),
                    ..Default::default()
                },
                BorderRadius::all(Val::Percent(50.0)),
                BackgroundColor(PAUSE_ICON_COLOR),
                // Inherit visibility from parent, so it's hidden initially.
                Visibility::Inherited,
                ZIndex(2),
            ));
        });

    // Load the sprite sheet and texture atlas for the number font.
    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Number.sprite");
    loading_assets.handles.push(texture_handle.clone().into());

    // Load the texture atlas layout for the number font.
    let texture_atlas_handle: Handle<TextureAtlasLayout> =
        asset_server.load("fonts/ImgFont_Number.atlas");
    // Add the handle to the list of assets to track for loading progress.
    loading_assets
        .handles
        .push(texture_atlas_handle.clone().into());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(1.5),
                left: Val::Vw(1.5),
                width: Val::Vw(30.0),
                height: Val::Vw(7.5),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..Default::default()
            },
            // This Animator component will handle the slide-in animation.
            // The score UI slides in from the top.
            Animator::new(Tween::new(
                EaseFunction::SmoothStep,
                Duration::from_secs_f32(UI_ANIMATION_DURATION),
                UiPositionLens {
                    start: UiRect {
                        top: Val::Vh(-20.0),
                        left: Val::Vw(1.5),
                        bottom: Val::Auto,
                        right: Val::Auto,
                    },
                    end: UiRect {
                        top: Val::Vh(1.5),
                        left: Val::Vw(1.5),
                        bottom: Val::Auto,
                        right: Val::Auto,
                    },
                },
            ))
            .with_state(AnimatorState::Paused), // Start the animation in a paused state.
            InGameStateEntity,
            Visibility::Hidden,
            UI::Score, // Marker component for the score display.
            ZIndex(1),
        ))
        .with_children(|parent| {
            // Spawn the 100,000s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace100000s,
            ));

            // Spawn the 10,000s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace10000s,
            ));

            // Spawn the 1,000s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace1000s,
            ));

            // Spawn the 100s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace100s,
            ));

            // Spawn the 10s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace10s,
            ));

            // Spawn the 1s place digit.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace1s,
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
                width: Val::Vw(30.0),
                height: Val::Vw(7.5),
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            Animator::new(Tween::new(
                // Define the animation for the fuel gauge sliding in from the bottom.
                EaseFunction::SmoothStep,
                Duration::from_secs_f32(UI_ANIMATION_DURATION),
                UiPositionLens {
                    start: UiRect {
                        top: Val::Auto,
                        left: Val::Auto,
                        bottom: Val::Vh(-20.0),
                        right: Val::Vw(3.0),
                    },
                    end: UiRect {
                        top: Val::Auto,
                        left: Val::Auto,
                        bottom: Val::Vh(1.5),
                        right: Val::Vw(3.0),
                    },
                },
            ))
            .with_state(AnimatorState::Paused),
            InGameStateEntity,
            Visibility::Hidden, // Hide the entire UI hierarchy initially.
            UI::Fuel,           // Marker component.
            ZIndex(1),
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
                    Visibility::Inherited, // Inherit visibility from parent.
                    ZIndex(2),             // Ensure the border is drawn above the gauge bar.
                ))
                .with_children(|parent| {
                    // Create the actual fuel gauge bar that will change width.
                    // This node is a child of the background, so it appears inside it.
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0), // Starts full
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BackgroundColor(FUEL_GOOD_GAUGE_COLOR),
                        BorderRadius::all(Val::Percent(50.0)),
                        Visibility::Inherited, // Inherit visibility from parent.
                        ZIndex(2),             // Drawn below the border.
                        FuelGauge,             // Marker to identify this entity for updates.
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
                Visibility::Inherited, // Inherit visibility from parent.
                ZIndex(2),             // Ensure it's drawn on top.
                FuelDeco,              // Marker component.
            ));
        });
}

/// Creates the UI elements for the pause menu and resume countdown.
///
/// This includes the "Pause" title and the "3, 2, 1" countdown numbers.
/// All elements are spawned with `Visibility::Hidden` and are made visible
/// by the systems in the `pause` and `resume` modules.
fn create_pause_ui(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    loading_assets: &mut LoadingAssets,
) {
    // Load the texture for the "Pause" title.
    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Pause.sprite");
    loading_assets.handles.push(texture_handle.clone().into());

    // Load the texture for the "Resume" button text.
    let resume_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Resume.sprite");
    loading_assets.handles.push(resume_handle.clone().into());

    // Load the texture for the "Exit" button text.
    let exit_handle: Handle<Image> = asset_server.load("fonts/ImgFont_Exit.sprite");
    loading_assets.handles.push(exit_handle.clone().into());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(UI_PAUSE_BG_COLOR),
            InGameStateEntity,
            Visibility::Hidden,
            UI::PauseTitle, // Marker component for the pause menu title.
            ZIndex(5),
        ))
        .with_children(|parent| {
            // Spawn the image node for the "Pause" title.
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                PauseTitle, // Marker component for the pause menu title.
            ));

            parent.spawn((Node {
                width: Val::Percent(30.0),
                height: Val::Percent(1.5),
                ..Default::default()
            },));

            parent
                .spawn((
                    Node {
                        width: Val::Percent(30.0),
                        height: Val::Percent(8.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Baseline,
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    // Spawn the "Resume" button.
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(48.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(30.0)),
                            BackgroundColor(RESUME_BTN_COLOR),
                            Visibility::Inherited,
                            UI::ResumeButton, // Marker component for the resume button.
                            Button,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ImageNode::new(resume_handle),
                                Node {
                                    width: Val::Auto,
                                    height: Val::Percent(100.0),
                                    overflow: Overflow::hidden(),
                                    ..Default::default()
                                },
                                Visibility::Inherited,
                            ));
                        });

                    // A small, invisible spacer between the two buttons.
                    parent.spawn((
                        Node {
                            width: Val::Percent(4.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        Visibility::Hidden,
                    ));

                    // Spawn the "Exit" button.
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(48.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(30.0)),
                            BackgroundColor(EXIT_BTN_COLOR),
                            Visibility::Inherited,
                            UI::ExitButton, // Marker component for the exit button.
                            Button,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ImageNode::new(exit_handle),
                                Node {
                                    width: Val::Auto,
                                    height: Val::Percent(100.0),
                                    overflow: Overflow::hidden(),
                                    ..Default::default()
                                },
                                Visibility::Inherited,
                            ));
                        });
                });
        });

    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_1.sprite");
    loading_assets.handles.push(texture_handle.clone().into());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            UI::ResumeCount1,
            ZIndex(5),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_2.sprite");
    loading_assets.handles.push(texture_handle.clone().into());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            UI::ResumeCount2,
            ZIndex(5),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    let texture_handle: Handle<Image> = asset_server.load("fonts/ImgFont_3.sprite");
    loading_assets.handles.push(texture_handle.clone().into());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            InGameStateEntity,
            Visibility::Hidden,
            UI::ResumeCount3,
            ZIndex(5),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(texture_handle),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
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
/// This runs once when transitioning from `GameState::Loading` to the `GameState::Prepare`.
pub fn on_exit(
    mut commands: Commands,
    // Query for all entities that are part of the loading screen.
    load_entities: Query<Entity, With<InGameLoadStateEntity>>,
    // Query for all entities that are part of the main game.
    mut in_game_entities: Query<&mut Visibility, (With<InGameStateEntity>, Without<UI>)>,
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

/// A system that checks if all assets are loaded and transitions to the `Prepare` state.
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

    // If all assets are loaded, transition to the `Prepare` state.
    if all_loaded {
        next_state.set(GameState::Prepare);
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
