// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};

use crate::asset::sound::SystemVolume;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Ranking), (debug_label, show_interface))
            .add_systems(OnExit(GameState::Ranking), hide_interface)
            .add_systems(
                PreUpdate,
                handle_button_system.run_if(in_state(GameState::Ranking)),
            );
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

// --- CLEANUP SYSTEMS ---

fn hide_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::TitleLeaderBoard | UI::LeaderBoardBackButton => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
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
