use std::time::Duration;

// Import necessary Bevy modules.
use bevy::prelude::*;
use bevy_tweening::{Animator, AnimatorState, Tween, lens::UiPositionLens};

use super::*;

// --- SETUP SYSTEM ---

/// System that sets up the game world and UI elements.
pub fn on_setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
) {
    info!("Setting up game world...");
    setup_game_world(&mut commands, &asset_server);
    setup_game_ui(&mut commands, &asset_server);
    setup_pause_ui(&mut commands, &asset_server);
    setup_result_world(&mut commands, &asset_server);
    next_state.set(GameState::Prepare);
}

/// Sets up the initial game world by pre-spawning entities.
///
/// This system assumes all necessary assets have already been loaded during the `Loading` state.
/// Entities are spawned with `Visibility::Hidden` to prevent visual glitches, and will be made
/// visible when the game transitions to the `Prepare` state.
fn setup_game_world(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Pre-spawn the initial ground segments. They are created with `Visibility::Hidden` and will be made
    // visible when the game starts. This technique, often called "object pooling" or "pre-warming",
    // helps prevent a performance stutter that could occur if these entities were all spawned at once at the beginning of the game.
    let model = asset_server.load("models/Plane_0.hierarchy");
    for i in 0..7 {
        commands.spawn((
            SpawnModel(model.clone()),
            Transform::from_xyz(0.0, 0.0, DESPAWN_LOCATION + 30.0 * i as f32),
            InGameStateEntity,
            Visibility::Hidden,
            Ground,
        ));
    }

    // Pre-spawn the first toy train model.
    let model = asset_server.load("models/ToyTrain00.hierarchy");
    // This entity is hidden initially and will be made visible later.
    commands.spawn((
        SpawnModel(model),
        Transform::IDENTITY,
        InGameStateEntity,
        Visibility::Hidden,
        ToyTrain0,
    ));

    // Pre-spawn the second toy train model.
    let model = asset_server.load("models/ToyTrain01.hierarchy");
    commands
        .spawn((
            SpawnModel(model),
            Transform::IDENTITY,
            InGameStateEntity,
            Visibility::Hidden,
            ToyTrain1,
        ))
        .with_children(|parent| {
            // Spawn the character entity (Hikari). It will be parented to this toy train car.
            let clip = asset_server.load("animations/Hikari_InGame.anim");
            let model = asset_server.load("models/Hikari.hierarchy");
            parent.spawn((
                SpawnModel(model),
                AnimationClipHandle(clip),
                Transform::from_xyz(0.0, 0.8775, 0.0),
                Visibility::Inherited,
            ));
        });

    // Pre-spawn the third toy train model.
    let model = asset_server.load("models/ToyTrain02.hierarchy");
    commands
        .spawn((
            SpawnModel(model),
            Transform::default(),
            InGameStateEntity,
            Visibility::Hidden,
            ToyTrain2,
        ))
        .with_children(|parent| {
            // Spawn the character entity (Nozomi). It will be parented to this toy train car.
            let clip = asset_server.load("animations/Nozomi_InGame.anim");
            let model = asset_server.load("models/Nozomi.hierarchy");
            parent.spawn((
                SpawnModel(model),
                AnimationClipHandle(clip),
                Transform::from_xyz(0.0, 0.5, 0.375),
                Visibility::Inherited,
            ));
        });
}

/// Sets up the in-game UI elements.
///
/// This system assumes all necessary UI assets (textures, fonts) have already been loaded
/// during the `Loading` state. The UI elements are spawned here and initially hidden,
/// to be made visible and animated when the game transitions to the `InGame` state.
fn setup_game_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Spawn the root node for the "Start" UI element.
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
            UI::Start,
        ))
        .with_children(|parent| {
            // Spawn the image node for the "Start" message.
            let texture = asset_server.load("fonts/ImgFont_Start.sprite");
            parent.spawn((
                ImageNode::new(texture),
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
            let texture = asset_server.load("fonts/ImgFont_Finish.sprite");
            parent.spawn((
                ImageNode::new(texture),
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

    // Spawn the score UI element.
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
            .with_state(AnimatorState::Paused),
            InGameStateEntity,
            Visibility::Hidden,
            UI::Score, // Marker component for the score display.
            ZIndex(1),
        ))
        .with_children(|parent| {
            let texture = asset_server.load("fonts/ImgFont_Number.sprite");
            let atlas = asset_server.load("fonts/ImgFont_Number.atlas");

            // Spawn the 100,000s place digit.
            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
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
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
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
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
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
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
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
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
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
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited, // Inherit visibility from parent.
                ScoreSpace1s,
            ));
        });

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
            let texture = asset_server.load("textures/Train_Icon.sprite");
            parent.spawn((
                ImageNode::new(texture).with_color(FUEL_COLOR),
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

/// Sets up the UI elements for the pause menu and resume countdown.
///
/// This includes the "Pause" title and the "3, 2, 1" countdown numbers.
/// All elements are spawned with `Visibility::Hidden` and are made visible
/// by the systems in the `pause` and `resume` modules.
fn setup_pause_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Spawns the root node for the pause menu UI.
    // This node acts as a container for the pause title and buttons.
    // It is initially hidden and will be made visible when the game is paused.
    // It has a semi-transparent background to dim the game world when paused.
    //
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
            let texture = asset_server.load("fonts/ImgFont_Pause.sprite");
            parent.spawn((
                ImageNode::new(texture),
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
                            let texture = asset_server.load("fonts/ImgFont_Resume.sprite");
                            parent.spawn((
                                ImageNode::new(texture),
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
                            let texture = asset_server.load("fonts/ImgFont_Exit.sprite");
                            parent.spawn((
                                ImageNode::new(texture),
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

    // Spawn the UI for the "3, 2, 1" resume countdown.
    let texture = asset_server.load("fonts/ImgFont_1.sprite");
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
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    let texture = asset_server.load("fonts/ImgFont_2.sprite");
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
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    let texture = asset_server.load("fonts/ImgFont_3.sprite");
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
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });
}

/// Sets up the game world for the result display.
/// This system assumes all necessary assets have already been loaded during the `Loading` state.
fn setup_result_world(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Pre-spawn the ground model specifically for the result display area.
    let model = asset_server.load("models/Plane_999.hierarchy");
    commands.spawn((
        SpawnModel(model.clone()),
        Transform::IDENTITY,
        Visibility::Hidden,
        ResultStateEntity,
    ));

    commands
        .spawn((
            Transform::from_xyz(-3.0, 0.0, 0.0)
                .with_rotation(quat(0.0, -0.7071068, 0.0, 0.7071068)),
            Rotate {
                radian_per_sec: 360f32.to_radians() / 1.0,
                ..Default::default()
            },
            Visibility::Hidden,
            ResultStateEntity,
        ))
        .with_children(|parent| {
            // Pre-spawn Hikari character for the result screen.
            let clip = asset_server.load("animations/Hikari_Victory_Start_Interaction.anim");
            let model = asset_server.load("models/Hikari.hierarchy");
            parent.spawn((
                SpawnModel(model),
                Transform::IDENTITY,
                AnimationClipHandle(clip),
                Visibility::Inherited,
                ResultStateEntity,
            ));

            // Pre-spawn Nozomi character for the result screen.
            let clip = asset_server.load("animations/Nozomi_Victory_Start_Interaction.anim");
            let model = asset_server.load("models/Nozomi.hierarchy");
            parent.spawn((
                SpawnModel(model),
                AnimationClipHandle(clip),
                Transform::IDENTITY,
                Visibility::Inherited,
                ResultStateEntity,
            ));
        });
}

// --- CLEANUP SYSTEM ---

/// A system that despawns all entities marked with `LoadingStateEntity`.
pub fn remove_loading_state_entities(
    mut commands: Commands,
    query: Query<Entity, With<LoadingStateEntity>>,
) {
    // Despawn all entities associated with the loading screen (UI camera, text, loading bar).
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    info!("Loading complete! Starting game...");
}
