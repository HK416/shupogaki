// Import necessary Bevy modules.
use bevy::{audio::Volume, ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::asset::{sound::SystemVolume, spawner::TranslatableText};

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register systems to run when entering the `GameState::Initialize` state.
            .add_systems(
                OnEnter(GameState::Initialize),
                (debug_label, play_loading_sound, spawn_entities),
            )
            // Register a cleanup system to run when exiting the `GameState::Initialize` state.
            .add_systems(OnExit(GameState::Initialize), remove_resource)
            // Register systems to run every frame while in the `GameState::Initialize` state.
            .add_systems(
                Update,
                (
                    handle_spawn_request,
                    check_loading_progress,
                    update_loading_bar,
                )
                    .run_if(in_state(GameState::Initialize)),
            );
    }
}

// --- SETUP SYSTEM ---

/// Prints a debug message to the console indicating the current game state.
fn debug_label() {
    info!("Current State: Initialize");
}

#[cfg(not(target_arch = "wasm32"))]
fn play_loading_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_loading_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

/// Spawns persistent UI entities, such as the options modal, that will be used across different scenes.
/// These entities are initially hidden and are tracked via the `LoadingEntities` resource.
fn spawn_entities(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_entities = LoadingEntities::default();

    // Spawn the root node for the options modal.
    let entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            // Mark this entity with a request to be processed later.
            SpawnRequest,
        ))
        .with_children(|parent| {
            // Create the main modal panel.
            parent
                .spawn((
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(50.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::WHITE.with_alpha(0.8)),
                    BorderRadius::all(Val::Percent(6.0)),
                    Visibility::Hidden, // The modal is hidden by default.
                    UI::OptionModal,
                ))
                .with_children(|parent| {
                    // Add UI elements to the modal.
                    add_vertical_space(parent, 10.0);
                    add_bgm_volume_controller(parent, &asset_server, 100.0, 10.0);
                    add_vertical_space(parent, 4.0);
                    add_sfx_volume_controller(parent, &asset_server, 100.0, 10.0);
                    add_vertical_space(parent, 4.0);
                    add_voice_volume_controller(parent, &asset_server, 100.0, 10.0);
                    add_vertical_space(parent, 10.0);
                    add_locale_button(parent, &asset_server, 100.0, 16.0);
                    add_vertical_space(parent, 10.0);
                    add_back_button(parent, &asset_server, 100.0, 16.0);
                    add_vertical_space(parent, 4.0);
                });
        })
        .id();

    // Add the spawned entity's ID to the list of entities to track for loading.
    loading_entities.handles.push(entity);

    commands.insert_resource(loading_entities);
}

/// Helper function to add a vertical spacer node for UI layout.
fn add_vertical_space<'a>(parent: &mut RelatedSpawnerCommands<'a, ChildOf>, h: f32) {
    parent.spawn(Node {
        height: Val::Percent(h),
        ..Default::default()
    });
}

/// Helper function to add a horizontal spacer node for UI layout.
fn add_horizontal_space<'a>(parent: &mut RelatedSpawnerCommands<'a, ChildOf>, w: f32) {
    parent.spawn(Node {
        width: Val::Percent(w),
        ..Default::default()
    });
}

/// Helper function to build and add the BGM volume control UI (label, slider, value).
fn add_bgm_volume_controller<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
    w: f32,
    h: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(w),
            height: Val::Percent(h),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("BGM"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        TranslatableText("background_music".into()),
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Hidden,
                        UI::BgmLabel,
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Percent(15.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(SLIDER_RAIL_COLOR),
                    Visibility::Hidden,
                    UI::SliderRail,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            align_content: AlignContent::Center,
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    left: Val::Percent(80.0),
                                    ..Default::default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            left: Val::VMin(-1.5),
                                            width: Val::VMin(3.0),
                                            height: Val::VMin(3.0),
                                            ..Default::default()
                                        },
                                        BackgroundColor(SLIDER_HANDLE_COLOR),
                                        BorderRadius::all(Val::Px(12.0)),
                                        UI::BgmVolumeCursor,
                                        Visibility::Hidden,
                                        Button,
                                    ));
                                });
                        });
                });

            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("80"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Hidden,
                        UI::BgmVolume,
                    ));
                });
        });
}

/// Helper function to build and add the SFX volume control UI (label, slider, value).
fn add_sfx_volume_controller<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
    w: f32,
    h: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(w),
            height: Val::Percent(h),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("SFX"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        TranslatableText("sound_effect".into()),
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Hidden,
                        UI::SfxLabel,
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Percent(15.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(SLIDER_RAIL_COLOR),
                    Visibility::Hidden,
                    UI::SliderRail,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            align_content: AlignContent::Center,
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    left: Val::Percent(80.0),
                                    ..Default::default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            left: Val::VMin(-1.5),
                                            width: Val::VMin(3.0),
                                            height: Val::VMin(3.0),
                                            ..Default::default()
                                        },
                                        BackgroundColor(SLIDER_HANDLE_COLOR),
                                        BorderRadius::all(Val::Px(12.0)),
                                        UI::SfxVolumeCursor,
                                        Visibility::Hidden,
                                        Button,
                                    ));
                                });
                        });
                });

            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("80"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        ResizableFont::vertical(1280.0, 42.0),
                        TextColor::BLACK,
                        Node::default(),
                        Visibility::Hidden,
                        UI::SfxVolume,
                    ));
                });
        });
}

/// Helper function to build and add the Voice volume control UI (label, slider, value).
fn add_voice_volume_controller<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
    w: f32,
    h: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(w),
            height: Val::Percent(h),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("Voice"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        TranslatableText("voice".into()),
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Hidden,
                        UI::VoiceLabel,
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Percent(15.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(SLIDER_RAIL_COLOR),
                    Visibility::Hidden,
                    UI::SliderRail,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            align_content: AlignContent::Center,
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    left: Val::Percent(80.0),
                                    ..Default::default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            left: Val::VMin(-1.5),
                                            width: Val::VMin(3.0),
                                            height: Val::VMin(3.0),
                                            ..Default::default()
                                        },
                                        BackgroundColor(SLIDER_HANDLE_COLOR),
                                        BorderRadius::all(Val::Px(12.0)),
                                        UI::VoiceVolumeCursor,
                                        Visibility::Hidden,
                                        Button,
                                    ));
                                });
                        });
                });

            parent
                .spawn(Node {
                    width: Val::Percent(30.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("80"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Hidden,
                        UI::VoiceVolume,
                    ));
                });
        });
}

/// Helper function to build and add the language selection buttons.
fn add_locale_button<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
    w: f32,
    h: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(w),
            height: Val::Percent(h),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(20.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Percent(0.5)),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(LANGUAGE_BTN_COLOR),
                    BorderRadius::all(Val::Percent(20.0)),
                    Visibility::Hidden,
                    UI::LanguageEn,
                    Button,
                ))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("English"),
                        TextFont::from_font(font.clone()),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Inherited,
                    ));
                });

            add_horizontal_space(parent, 10.0);

            parent
                .spawn((
                    Node {
                        width: Val::Percent(20.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Percent(0.5)),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(LANGUAGE_BTN_COLOR),
                    BorderRadius::all(Val::Percent(20.0)),
                    Visibility::Hidden,
                    UI::LanguageJa,
                    Button,
                ))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("日本語"),
                        TextFont::from_font(font.clone()),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Inherited,
                    ));
                });

            add_horizontal_space(parent, 10.0);

            parent
                .spawn((
                    Node {
                        width: Val::Percent(20.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Percent(0.5)),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(LANGUAGE_BTN_COLOR),
                    BorderRadius::all(Val::Percent(20.0)),
                    Visibility::Hidden,
                    UI::LanguageKo,
                    Button,
                ))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("한국어"),
                        TextFont::from_font(font.clone()),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Inherited,
                    ));
                });
        });
}

/// Helper function to build and add the 'Back' button for the options modal.
fn add_back_button<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
    w: f32,
    h: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(w),
            height: Val::Percent(h),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(30.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(BACK_BTN_COLOR),
                    BorderRadius::all(Val::Percent(20.0)),
                    Visibility::Hidden,
                    UI::BackButton,
                    Button,
                ))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("Back"),
                        TextFont::from_font(font.clone()),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        TranslatableText("back".into()),
                        ResizableFont::vertical(1280.0, 42.0),
                        Node::default(),
                        Visibility::Inherited,
                    ));
                });
        });
}

// --- CLEANUP SYSTEMS ---

/// Removes the `LoadingEntities` resource when exiting the `Initialize` state.
fn remove_resource(mut commands: Commands) {
    commands.remove_resource::<LoadingEntities>();
}

// --- UPDATE SYSTEMS ---

/// Processes entities marked with `SpawnRequest`.
/// It removes the `SpawnRequest` component and adds `OptionStateRoot` to finalize the entity's setup.
fn handle_spawn_request(mut commands: Commands, query: Query<Entity, Added<SpawnRequest>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<SpawnRequest>()
            .insert(OptionStateRoot);
    }
}

/// Checks if all requested entities have been spawned and processed.
/// Once all entities are ready, it transitions to the `GameState::LoadTitle`.
fn check_loading_progress(
    mut next_state: ResMut<NextState<GameState>>,
    loading_entitis: Res<LoadingEntities>,
    query: Query<(), With<SpawnRequest>>,
) {
    // Check if all entities in the loading list no longer have the `SpawnRequest` component.
    let all_loaded = loading_entitis
        .handles
        .iter()
        .all(|entity| !query.contains(*entity));

    if all_loaded {
        // Transition to the next state once all entities are initialized.
        next_state.set(GameState::LoadTitle);
    }
}

/// Updates the loading bar's width based on the progress of entity spawning.
fn update_loading_bar(
    loading_entitis: Res<LoadingEntities>,
    request_query: Query<(), With<SpawnRequest>>,
    mut query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = query.single_mut() {
        // Count how many entities have been fully processed (no longer have `SpawnRequest`).
        let loaded_count = loading_entitis
            .handles
            .iter()
            .filter(|&&entity| !request_query.contains(entity))
            .count();

        let total_count = loading_entitis.handles.len();
        // Calculate the progress percentage.
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0 // Avoid division by zero.
        };

        // Update the width of the loading bar node.
        node.width = Val::Percent(progress * 100.0);
    }
}
