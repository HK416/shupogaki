// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};

use crate::asset::{
    animation::AnimationClipHandle,
    sound::SystemVolume,
    spawner::{SpawnModel, TranslatableText},
};

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitInGame),
            (debug_label, play_loading_sound, spawn_entities),
        )
        .add_systems(OnExit(GameState::InitInGame), remove_resource)
        .add_systems(
            Update,
            (
                handle_spawn_request,
                check_loading_progress,
                update_loading_bar,
            )
                .run_if(in_state(GameState::InitInGame)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: InitInGame");
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

fn spawn_entities(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_entities = LoadingEntities::default();
    spawn_in_game_entities(&mut commands, &asset_server, &mut loading_entities);
    spawn_in_game_ui_entities(&mut commands, &asset_server, &mut loading_entities);
    spawn_pause_ui_entities(&mut commands, &asset_server, &mut loading_entities);
    commands.insert_resource(loading_entities);
}

fn spawn_in_game_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    let entity = commands
        .spawn((
            Transform::from_xyz(LANE_LOCATIONS[NUM_LANES / 2], 0.0, PLAYER_MAX_Z_POS),
            Collider::Aabb {
                offset: Vec3::new(0.0, 0.5, -1.5),
                size: Vec3::new(0.9, 1.0, 3.6),
            },
            SpawnRequest,
            Player,
        ))
        .id();
    loading_entities.handles.push(entity);

    let model = asset_server.load(MODEL_PATH_PLANE_0);
    let mut plane_location = DESPAWN_LOCATION;
    while plane_location <= SPAWN_LOCATION {
        let entity = commands
            .spawn((
                SpawnModel(model.clone()),
                Transform::from_xyz(0.0, 0.0, plane_location),
                Visibility::Hidden,
                SpawnRequest,
                Ground,
            ))
            .id();
        loading_entities.handles.push(entity);
        plane_location += PLANE_SPAWN_INTERVAL;
    }

    let model = asset_server.load(MODEL_PATH_TOY_TRAIN_00);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::IDENTITY,
            Visibility::Hidden,
            SpawnRequest,
            ToyTrain0,
        ))
        .id();
    loading_entities.handles.push(entity);

    let model = asset_server.load(MODEL_PATH_TOY_TRAIN_01);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::IDENTITY,
            Visibility::Hidden,
            SpawnRequest,
            ToyTrain1,
        ))
        .with_children(|parent| {
            let clip = asset_server.load(ANIM_PATH_HIKARI_IN_GAME);
            let model = asset_server.load(MODEL_PATH_HIKARI);
            parent.spawn((
                SpawnModel(model),
                AnimationClipHandle(clip),
                Transform::from_xyz(0.0, 0.8775, 0.0),
                Visibility::Inherited,
                InGameStateEntity,
                Hikari,
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    let model = asset_server.load(MODEL_PATH_TOY_TRAIN_02);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::IDENTITY,
            Visibility::Hidden,
            SpawnRequest,
            ToyTrain2,
        ))
        .with_children(|parent| {
            let clip = asset_server.load(ANIM_PATH_NOZOMI_IN_GAME);
            let model = asset_server.load(MODEL_PATH_NOZOMI);
            parent.spawn((
                SpawnModel(model),
                AnimationClipHandle(clip),
                Transform::from_xyz(0.0, 0.5, 0.375),
                Visibility::Inherited,
                InGameStateEntity,
                Nozomi,
            ));
        })
        .id();
    loading_entities.handles.push(entity);
}

fn spawn_in_game_ui_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    // --- Start Label ---
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
            SpawnRequest,
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_START);
            parent.spawn((
                ImageNode::new(texture).with_color(Color::WHITE.with_alpha(0.0)),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Hidden,
                UI::StartLabel,
                ZIndex(4),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Finish Label ---
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
            SpawnRequest,
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_FINISH);
            parent.spawn((
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Hidden,
                UI::FinishLabel,
                ZIndex(4),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Pause Button ---
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(1.5),
                right: Val::Vw(1.5),
                width: Val::Vw(4.5),
                height: Val::Vw(4.5),
                ..Default::default()
            },
            BackgroundColor(PAUSE_BTN_COLOR),
            BorderRadius::all(Val::Percent(30.0)),
            SpawnRequest,
            Visibility::Hidden,
            UI::PauseButton,
            ZIndex(1),
            Button,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    top: Val::Percent(20.0),
                    left: Val::Percent(30.0),
                    width: Val::Percent(15.0),
                    height: Val::Percent(60.0),
                    ..Default::default()
                },
                BorderRadius::all(Val::Percent(50.0)),
                BackgroundColor(PAUSE_ICON_COLOR),
                Visibility::Inherited,
                ZIndex(2),
            ));

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
                Visibility::Inherited,
                ZIndex(2),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Score ---
    let entity = commands
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
            SpawnRequest,
            Visibility::Hidden,
            UI::Score,
            ZIndex(1),
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_NUMBER);
            let atlas = asset_server.load(ATLAS_PATH_NUMBER);

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace100000s,
            ));

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace10000s,
            ));

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace1000s,
            ));

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace100s,
            ));

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace10s,
            ));

            parent.spawn((
                ImageNode::from_atlas_image(texture.clone(), TextureAtlas::from(atlas.clone())),
                Node {
                    width: Val::Auto,
                    height: Val::Percent(80.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScoreSpace1s,
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Fuel ---
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Vh(1.5),
                right: Val::Vw(3.0),
                width: Val::Vw(30.0),
                height: Val::Vw(7.5),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            SpawnRequest,
            Visibility::Hidden,
            UI::Fuel,
            ZIndex(1),
        ))
        .with_children(|parent| {
            // --- Fuel Deco ---
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(75.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    let texture = asset_server.load(TEXTURE_PATH_TRAIN_ICON);
                    parent.spawn((
                        ImageNode::new(texture).with_color(FUEL_COLOR),
                        Node {
                            top: Val::Percent(12.5),
                            left: Val::Px(0.0),
                            width: Val::Auto,
                            height: Val::Percent(90.0),
                            aspect_ratio: Some(2.0),
                            ..Default::default()
                        },
                        Visibility::Inherited,
                        ZIndex(2),
                        FuelDeco,
                    ));
                });

            // --- Fuel Gauge ---
            parent
                .spawn((
                    Node {
                        bottom: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(15.0),
                        border: UiRect::all(Val::Percent(1.0)),
                        ..Default::default()
                    },
                    BackgroundColor(FUEL_COLOR),
                    BorderColor(FUEL_COLOR),
                    BorderRadius::all(Val::Percent(50.0)),
                    Visibility::Inherited,
                    ZIndex(2),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BackgroundColor(FUEL_GOOD_GAUGE_COLOR),
                        BorderRadius::all(Val::Percent(50.0)),
                        Visibility::Inherited,
                        ZIndex(3),
                        FuelGauge,
                    ));
                });
        })
        .id();
    loading_entities.handles.push(entity);
}

fn spawn_pause_ui_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    // --- Count 1 ---
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
            SpawnRequest,
            Visibility::Hidden,
            UI::ResumeCount1,
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_NUM_1);
            parent.spawn((
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ZIndex(5),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Count 2 ---
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
            SpawnRequest,
            Visibility::Hidden,
            UI::ResumeCount2,
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_NUM_2);
            parent.spawn((
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ZIndex(5),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    // --- Count 3 ---
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
            SpawnRequest,
            Visibility::Hidden,
            UI::ResumeCount3,
        ))
        .with_children(|parent| {
            let texture = asset_server.load(FONT_PATH_NUM_3);
            parent.spawn((
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(15.0),
                    height: Val::Vw(15.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                ZIndex(5),
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    let entity = commands
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
            BackgroundColor(PAUSE_BG_COLOR),
            SpawnRequest,
            Visibility::Hidden,
            UI::Pause,
            ZIndex(5),
        ))
        .with_children(|parent| {
            // --- Title Label ---
            let texture = asset_server.load(FONT_PATH_PAUSE);
            parent.spawn((
                ImageNode::new(texture),
                Node {
                    width: Val::Vw(40.0),
                    height: Val::Auto,
                    aspect_ratio: Some(2.66666),
                    ..Default::default()
                },
                Visibility::Inherited,
                PauseTitle,
            ));

            // --- Space ---
            parent.spawn(Node {
                height: Val::Percent(1.5),
                ..Default::default()
            });

            // --- Buttons ---
            parent
                .spawn((
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Percent(8.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    // --- Resume Button ---
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(30.0)),
                            BackgroundColor(RESUME_BTN_COLOR),
                            Visibility::Inherited,
                            UI::ResumeButton,
                            Button,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Resume"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor(Color::BLACK),
                                TranslatableText("resume".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 42.0,
                                },
                                Node::default(),
                                Visibility::Inherited,
                            ));
                        });

                    // --- Space ---
                    parent.spawn(Node {
                        width: Val::Percent(10.0),
                        ..Default::default()
                    });

                    // --- Option Button ---
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(30.0)),
                            BackgroundColor(OPTION_BTN_COLOR),
                            Visibility::Inherited,
                            UI::OptionButton,
                            Button,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Settings"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor(Color::BLACK),
                                TranslatableText("option".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 42.0,
                                },
                                Node::default(),
                                Visibility::Inherited,
                            ));
                        });

                    // --- Space ---
                    parent.spawn(Node {
                        width: Val::Percent(10.0),
                        ..Default::default()
                    });

                    // --- Exit Button ---
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(30.0)),
                            BackgroundColor(EXIT_BTN_COLOR),
                            Visibility::Inherited,
                            UI::InGameExitButton,
                            Button,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Exit"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor(Color::BLACK),
                                TranslatableText("exit".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 42.0,
                                },
                                Node::default(),
                                Visibility::Inherited,
                            ));
                        });
                });
        })
        .id();
    loading_entities.handles.push(entity);
}

// --- CLEANUP SYSTEMS ---

fn remove_resource(mut commands: Commands) {
    commands.remove_resource::<LoadingEntities>();
}

// --- UPDATE SYSTEMS ---

fn handle_spawn_request(mut commands: Commands, query: Query<Entity, Added<SpawnRequest>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<SpawnRequest>()
            .insert(InGameStateRoot);
    }
}

fn check_loading_progress(
    mut next_state: ResMut<NextState<GameState>>,
    loading_entitis: Res<LoadingEntities>,
    query: Query<(), With<SpawnRequest>>,
) {
    let all_loaded = loading_entitis
        .handles
        .iter()
        .all(|entity| !query.contains(*entity));

    if all_loaded {
        next_state.set(GameState::InitResult);
    }
}

fn update_loading_bar(
    loading_entitis: Res<LoadingEntities>,
    request_query: Query<(), With<SpawnRequest>>,
    mut query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = query.single_mut() {
        let loaded_count = loading_entitis
            .handles
            .iter()
            .filter(|&&entity| !request_query.contains(entity))
            .count();

        let total_count = loading_entitis.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0
        };

        node.width = Val::Percent(progress * 100.0);
    }
}
