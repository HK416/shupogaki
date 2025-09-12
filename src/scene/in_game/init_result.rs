// Import necessary Bevy modules.
use bevy::{prelude::*, render::view::NoFrustumCulling};

use crate::asset::{
    animation::AnimationClipHandle,
    spawner::{SpawnModel, TranslatableText},
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitResult),
            (debug_label, spawn_entities),
        )
        .add_systems(
            OnExit(GameState::InitResult),
            (remove_entities, remove_resource),
        )
        .add_systems(
            Update,
            (
                disable_frustum_culling,
                handle_spawn_request,
                check_loading_progress,
                update_loading_bar,
            )
                .run_if(in_state(GameState::InitResult)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: InitResult");
}

fn spawn_entities(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_entities = LoadingEntities::default();
    spawn_result_entities(&mut commands, &asset_server, &mut loading_entities);
    spawn_result_ui_entities(&mut commands, &asset_server, &mut loading_entities);
    commands.insert_resource(loading_entities);
}

fn spawn_result_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    let model = asset_server.load(MODEL_PATH_PLANE_999);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            SpawnRequest,
        ))
        .id();
    loading_entities.handles.push(entity);

    let clip = asset_server.load(ANIM_PATH_HIKARI_VICTORY_START);
    let model = asset_server.load(MODEL_PATH_HIKARI);
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_translation(result::HIKARI_POSITION)
                .looking_to(result::STUDENT_DIRECTION, Vec3::Y),
            Visibility::Hidden,
            SpawnRequest,
            ResultStateEntity,
            Hikari,
        ))
        .id();
    loading_entities.handles.push(entity);

    let clip = asset_server.load(ANIM_PATH_NOZOMI_VICTORY_START);
    let model = asset_server.load(MODEL_PATH_NOZOMI);
    let entity = commands
        .spawn((
            SpawnModel(model),
            AnimationClipHandle(clip),
            Transform::from_translation(result::NOZOMI_POSITION)
                .looking_to(result::STUDENT_DIRECTION, Vec3::Y),
            Visibility::Hidden,
            SpawnRequest,
            ResultStateEntity,
            Nozomi,
        ))
        .id();
    loading_entities.handles.push(entity);
}

fn spawn_result_ui_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    let entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            SpawnRequest,
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(35.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },))
                .with_children(|parent| {
                    parent.spawn(Node {
                        height: Val::Percent(5.0),
                        ..Default::default()
                    });

                    parent
                        .spawn(Node {
                            left: Val::Percent(5.0),
                            width: Val::Percent(100.0),
                            height: Val::Percent(10.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Game Result"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextShadow::default(),
                                TextColor(Color::WHITE.with_alpha(0.0)),
                                TranslatableText("result".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 84.0,
                                },
                                Node::default(),
                                Visibility::Hidden,
                                UI::ResultText,
                            ));
                        });

                    parent
                        .spawn((
                            Node {
                                left: Val::Percent(7.0),
                                width: Val::Percent(100.0),
                                height: Val::Percent(20.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BackgroundColor(Color::WHITE.with_alpha(0.0)),
                            BorderRadius::all(Val::Percent(10.0)),
                            Visibility::Hidden,
                            UI::ResultModal,
                        ))
                        .with_children(|parent| {
                            parent.spawn(Node {
                                height: Val::Percent(10.0),
                                ..Default::default()
                            });

                            parent
                                .spawn(Node {
                                    width: Val::Percent(80.0),
                                    height: Val::Percent(30.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            width: Val::Percent(50.0),
                                            height: Val::Percent(100.0),
                                            justify_content: JustifyContent::Start,
                                            align_content: AlignContent::Center,
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            let texture = asset_server.load(FONT_PATH_TIME);
                                            parent.spawn((
                                                ImageNode::new(texture)
                                                    .with_color(Color::WHITE.with_alpha(0.0)),
                                                Node {
                                                    height: Val::Percent(100.0),
                                                    aspect_ratio: Some(256.0 / 96.0),
                                                    ..Default::default()
                                                },
                                                Visibility::Hidden,
                                                UI::ResultImgFont,
                                            ));
                                        });

                                    parent
                                        .spawn(Node {
                                            width: Val::Percent(50.0),
                                            height: Val::Percent(74.0),
                                            justify_content: JustifyContent::Start,
                                            align_content: AlignContent::Center,
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                                            parent.spawn((
                                                Text::new("%M:%S"),
                                                TextFont::from_font(font),
                                                TextLayout::new_with_justify(JustifyText::Center),
                                                TextColor::BLACK,
                                                ResizableFont::vertical(1280.0, 54.0),
                                                Node::default(),
                                                Visibility::Hidden,
                                                UI::PlayTime,
                                            ));
                                        });
                                });

                            parent.spawn(Node {
                                height: Val::Percent(1.0),
                                ..Default::default()
                            });

                            parent
                                .spawn(Node {
                                    width: Val::Percent(80.0),
                                    height: Val::Percent(30.0),
                                    justify_content: JustifyContent::Center,
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            width: Val::Percent(50.0),
                                            height: Val::Percent(100.0),
                                            justify_content: JustifyContent::Start,
                                            align_content: AlignContent::Center,
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            let texture = asset_server.load(FONT_PATH_SCORE);
                                            parent.spawn((
                                                ImageNode::new(texture)
                                                    .with_color(Color::WHITE.with_alpha(0.0)),
                                                Node {
                                                    height: Val::Percent(100.0),
                                                    aspect_ratio: Some(256.0 / 80.0),
                                                    ..Default::default()
                                                },
                                                Visibility::Hidden,
                                                UI::ResultImgFont,
                                            ));
                                        });

                                    parent
                                        .spawn(Node {
                                            width: Val::Percent(50.0),
                                            height: Val::Percent(74.0),
                                            justify_content: JustifyContent::Start,
                                            align_content: AlignContent::Center,
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                                            parent.spawn((
                                                Text::new("0"),
                                                TextFont::from_font(font),
                                                TextLayout::new_with_justify(JustifyText::Center),
                                                TextColor::BLACK,
                                                ResizableFont::vertical(1280.0, 54.0),
                                                Node::default(),
                                                Visibility::Hidden,
                                                UI::GameScore,
                                            ));
                                        });
                                });
                        });

                    parent.spawn(Node {
                        height: Val::Percent(65.0),
                        ..Default::default()
                    });
                });

            parent.spawn(Node {
                width: Val::Percent(30.0),
                ..Default::default()
            });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(35.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|parent| {
                    parent.spawn(Node {
                        height: Val::Percent(20.0),
                        ..Default::default()
                    });

                    parent.spawn(Node {
                        height: Val::Percent(50.0),
                        ..Default::default()
                    });

                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(60.0),
                                height: Val::Percent(10.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(10.0)),
                            BackgroundColor(RESTART_BTN_COLOR.with_alpha(0.0)),
                            Visibility::Hidden,
                            UI::RestartButton,
                            Button,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Restart"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TranslatableText("restart".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 64.0,
                                },
                                TextColor::BLACK,
                                Node::default(),
                                Visibility::Inherited,
                            ));
                        });

                    parent.spawn(Node {
                        height: Val::Percent(4.0),
                        ..Default::default()
                    });

                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(60.0),
                                height: Val::Percent(10.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            BorderRadius::all(Val::Percent(10.0)),
                            BackgroundColor(EXIT_BTN_COLOR.with_alpha(0.0)),
                            Visibility::Hidden,
                            UI::ResultExitButton,
                            Button,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Exit"),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TranslatableText("exit".into()),
                                ResizableFont::Vertical {
                                    base: 1280.0,
                                    size: 64.0,
                                },
                                TextColor::BLACK,
                                Node::default(),
                                Visibility::Inherited,
                            ));
                        });

                    parent.spawn(Node {
                        height: Val::Percent(6.0),
                        ..Default::default()
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

fn remove_entities(mut commands: Commands, query: Query<Entity, With<LoadingStateRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// --- UPDATE SYSTEMS ---

fn disable_frustum_culling(mut commands: Commands, query: Query<Entity, Added<Mesh3d>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(NoFrustumCulling);
    }
}

fn handle_spawn_request(mut commands: Commands, query: Query<Entity, Added<SpawnRequest>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<SpawnRequest>()
            .insert(ResultStateRoot);
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
        next_state.set(GameState::PrepareInGame);
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
