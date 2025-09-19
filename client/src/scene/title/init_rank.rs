// Import necessary Bevy modules.
use bevy::{audio::Volume, ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::asset::{sound::SystemVolume, spawner::TranslatableText};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitRank),
            (debug_label, play_loading_sound, spawn_entities),
        )
        .add_systems(
            OnExit(GameState::InitRank),
            (remove_resource, remove_entities),
        )
        .add_systems(
            Update,
            (
                handle_spawn_request,
                check_loading_progress,
                update_loading_progress,
            )
                .run_if(in_state(GameState::InitRank)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: InitRank");
}

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

fn spawn_entities(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_entities = LoadingEntities::default();
    spawn_leader_board_ui_entities(&mut commands, &asset_server, &mut loading_entities);

    // --- Resource Insertion ---
    commands.insert_resource(loading_entities);
}

fn spawn_leader_board_ui_entities(
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
            BackgroundColor(Color::BLACK.with_alpha(0.6)),
            UI::TitleLeaderBoard,
            Visibility::Hidden,
            SpawnRequest,
        ))
        .with_children(|parent| {
            add_modal(parent, asset_server);
        })
        .id();
    loading_entities.handles.push(entity);
}

fn add_horizontal_space<'a>(parent: &mut RelatedSpawnerCommands<'a, ChildOf>, p: f32) {
    parent.spawn(Node {
        width: Val::Percent(p),
        ..Default::default()
    });
}

fn add_vertical_space<'a>(parent: &mut RelatedSpawnerCommands<'a, ChildOf>, p: f32) {
    parent.spawn(Node {
        height: Val::Percent(p),
        ..Default::default()
    });
}

fn add_modal<'a>(parent: &mut RelatedSpawnerCommands<'a, ChildOf>, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BorderRadius::all(Val::Percent(10.0)),
            BackgroundColor(Color::WHITE),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(84.0),
                        height: Val::Percent(6.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.3)),
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    add_table_label(parent, asset_server);
                });

            parent.spawn((
                Node {
                    width: Val::Percent(84.0),
                    height: Val::Percent(70.0),
                    ..Default::default()
                },
                Visibility::Inherited,
                TitleLeaderBoard,
            ));

            add_vertical_space(parent, 2.0);

            parent
                .spawn((
                    Node {
                        width: Val::Percent(76.0),
                        height: Val::Percent(12.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(BACK_BTN_COLOR),
                    BorderRadius::all(Val::Percent(10.0)),
                    Visibility::Inherited,
                    UI::LeaderBoardBackButton,
                    Button,
                ))
                .with_children(|parent| {
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("Back"),
                        TextFont::from_font(font),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::BLACK,
                        ResizableFont::vertical(1280.0, 52.0),
                        TranslatableText("back".into()),
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        Visibility::Inherited,
                    ));
                });
        });

    add_horizontal_space(parent, 50.0);
}

fn add_table_label<'a>(
    parent: &mut RelatedSpawnerCommands<'a, ChildOf>,
    asset_server: &AssetServer,
) {
    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
    parent
        .spawn((
            Node {
                width: Val::Percent(15.0),
                height: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Rank"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor::BLACK,
                ResizableFont::vertical(1280.0, 32.0),
                TranslatableText("rank".into()),
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    add_horizontal_space(parent, 7.5);
    parent
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Name"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor::BLACK,
                ResizableFont::vertical(1280.0, 32.0),
                TranslatableText("name".into()),
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });

    add_horizontal_space(parent, 7.5);
    parent
        .spawn((
            Node {
                width: Val::Percent(30.0),
                height: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Score"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor::BLACK,
                ResizableFont::vertical(1280.0, 32.0),
                TranslatableText("score".into()),
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Visibility::Inherited,
            ));
        });
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

fn handle_spawn_request(mut commands: Commands, query: Query<Entity, Added<SpawnRequest>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<SpawnRequest>()
            .insert(RankingStateRoot);
    }
}

fn check_loading_progress(
    mut next_state: ResMut<NextState<GameState>>,
    spawn_request_entities: Query<(), With<SpawnRequest>>,
) {
    if spawn_request_entities.is_empty() {
        next_state.set(GameState::Title);
    }
}

fn update_loading_progress(
    loading_entities: Res<LoadingEntities>,
    spawn_request_entities: Query<(), With<SpawnRequest>>,
    mut loading_bar_query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = loading_bar_query.single_mut() {
        let loaded_count = loading_entities
            .handles
            .iter()
            .filter(|&&entity| !spawn_request_entities.contains(entity))
            .count();

        let total_count = loading_entities.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0
        };

        node.width = Val::Percent(progress * 100.0);
    }
}
