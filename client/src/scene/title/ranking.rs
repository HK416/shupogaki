// Import necessary Bevy modules.
use bevy::{
    audio::Volume,
    prelude::*,
    tasks::{IoTaskPool, futures_lite::future},
};
use bevy_simple_scroll_view::{ScrollView, ScrollableContent};

use crate::{
    asset::{config::Configuration, sound::SystemVolume},
    net::{HttpClient, RankingEntry, RankingStatus, RankingTask},
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Ranking),
            (debug_label, show_interface, setup_ranking_status),
        )
        .add_systems(
            OnExit(GameState::Ranking),
            (hide_interface, remove_leader_board_entry),
        )
        .add_systems(
            PreUpdate,
            handle_button_system.run_if(in_state(GameState::Ranking)),
        )
        .add_systems(
            Update,
            update_leader_board
                .run_if(resource_changed::<RankingStatus>)
                .run_if(in_state(GameState::Ranking)),
        )
        .add_systems(Update, handle_ranking_response);
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Ranking");
}

fn show_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::TitleLeaderBoard | UI::LeaderBoardBackButton => *visibility = Visibility::Visible,
            _ => { /* empty */ }
        }
    }
}

fn setup_ranking_status(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config_assets: Res<Assets<Configuration>>,
    http_client: Res<HttpClient>,
    mut rankings: ResMut<RankingStatus>,
) {
    match &*rankings {
        RankingStatus::Success { timer, .. } => if timer.elapsed_secs() < 60.0 { /* empty */ },
        _ => {
            let handle = asset_server.load(CONFIG_PATH);
            if let Some(config) = config_assets.get(&handle) {
                let pool = IoTaskPool::get();
                let client = http_client.0.clone();
                let url = config.server_url.clone();

                let task = pool.spawn(async move {
                    let response = client.get(url).send().await?;
                    response.json::<Vec<RankingEntry>>().await
                });

                commands.spawn(RankingTask(task));

                *rankings = RankingStatus::Loading;
            } else {
                *rankings = RankingStatus::Failed("Ranking server not found!".into());
            }
        }
    };
}

// --- CLEANUP SYSTEMS ---

fn hide_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::TitleLeaderBoard | UI::LeaderBoardBackButton => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
    }
}

fn remove_leader_board_entry(
    mut commands: Commands,
    entry_entities: Query<Entity, With<LeaderBoardEntry>>,
) {
    for entity in entry_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// --- PREUPDATE SYSTEMS ---

#[allow(clippy::type_complexity)]
fn handle_button_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mut query: Query<
        (&UI, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (&ui, &interaction, mut color) in query.iter_mut() {
        match (ui, interaction) {
            (UI::LeaderBoardBackButton, Interaction::Hovered) => {
                color.0 = BACK_BTN_COLOR.darker(0.15);
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::LeaderBoardBackButton, Interaction::Pressed) => {
                color.0 = BACK_BTN_COLOR.darker(0.3);
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                next_state.set(GameState::Title);
            }
            (UI::LeaderBoardBackButton, Interaction::None) => {
                color.0 = BACK_BTN_COLOR;
            }
            _ => { /* empty */ }
        }
    }
}

fn play_button_sound_when_hovered(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

fn play_button_sound_when_pressed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_BACK)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

// --- UPDATE SYSTEMS ---

fn handle_ranking_response(
    mut commands: Commands,
    mut ranking_task_query: Query<(Entity, &mut RankingTask)>,
    mut rankings: ResMut<RankingStatus>,
) {
    for (entity, mut task) in ranking_task_query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(data) => {
                    *rankings = RankingStatus::Success {
                        timer: Timer::from_seconds(60.0, TimerMode::Once),
                        entires: data,
                    };
                }
                Err(e) => {
                    *rankings = RankingStatus::Failed(e.to_string());
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

fn update_leader_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rankings: Res<RankingStatus>,
    prev_entry_entities: Query<Entity, With<LeaderBoardEntry>>,
    leader_board_entities: Query<Entity, With<TitleLeaderBoard>>,
) {
    for entity in prev_entry_entities.iter() {
        commands.entity(entity).despawn();
    }

    if let Ok(entity) = leader_board_entities.single() {
        match &*rankings {
            RankingStatus::Loading => {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                ..Default::default()
                            },
                            Visibility::Inherited,
                            LeaderBoardEntry,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new("Loading..."),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor::BLACK,
                                ResizableFont::vertical(1280.0, 72.0),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                Visibility::Inherited,
                            ));
                        });
                });
            }
            RankingStatus::Success { entires, .. } => {
                let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                build_leader_board(entity, &font, &mut commands, entires);
            }
            RankingStatus::Failed(message) => {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                ..Default::default()
                            },
                            Visibility::Inherited,
                            LeaderBoardEntry,
                        ))
                        .with_children(|parent| {
                            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                            parent.spawn((
                                Text::new(message),
                                TextFont::from_font(font),
                                TextLayout::new_with_justify(JustifyText::Center),
                                TextColor::BLACK,
                                ResizableFont::vertical(1280.0, 48.0),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                Visibility::Inherited,
                            ));
                        });
                });
            }
        }
    }
}

fn build_leader_board(
    entity: Entity,
    font: &Handle<Font>,
    commands: &mut Commands,
    entries: &[RankingEntry],
) {
    #[cfg(not(feature = "no-debuging-assert"))]
    assert!(entries.is_sorted(), "Unsorted entries!");

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..Default::default()
                },
                Visibility::Inherited,
                ScrollView::default(),
                LeaderBoardEntry,
            ))
            .with_children(|parent| {
                parent
                    .spawn((ScrollableContent::default(), Visibility::Inherited))
                    .with_children(|parent| {
                        for (i, entry) in entries.iter().enumerate() {
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Vh(5.0),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::Center,
                                        align_content: AlignContent::Center,
                                        ..Default::default()
                                    },
                                    BackgroundColor(if i % 2 == 0 {
                                        Color::WHITE
                                    } else {
                                        Color::BLACK.with_alpha(0.1)
                                    }),
                                    Visibility::Inherited,
                                ))
                                .with_children(|parent| {
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
                                                Text::new(format!("{}", i + 1)),
                                                TextFont::from_font(font.clone()),
                                                TextLayout::new_with_justify(JustifyText::Center),
                                                TextColor::BLACK,
                                                ResizableFont::vertical(1280.0, 28.0),
                                                Node {
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..Default::default()
                                                },
                                                Visibility::Inherited,
                                            ));
                                        });
                                    parent.spawn(Node {
                                        width: Val::Percent(7.5),
                                        ..Default::default()
                                    });

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
                                                Text::new(&entry.name),
                                                TextFont::from_font(font.clone()),
                                                TextLayout::new_with_justify(JustifyText::Center),
                                                TextColor::BLACK,
                                                ResizableFont::vertical(1280.0, 28.0),
                                                Node {
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..Default::default()
                                                },
                                                Visibility::Inherited,
                                            ));
                                        });

                                    parent.spawn(Node {
                                        width: Val::Percent(7.5),
                                        ..Default::default()
                                    });

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
                                                Text::new(entry.score.to_string()),
                                                TextFont::from_font(font.clone()),
                                                TextLayout::new_with_justify(JustifyText::Center),
                                                TextColor::BLACK,
                                                ResizableFont::vertical(1280.0, 28.0),
                                                Node {
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..Default::default()
                                                },
                                                Visibility::Inherited,
                                            ));
                                        });
                                });
                        }
                    });
            });
    });
}
