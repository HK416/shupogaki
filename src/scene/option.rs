// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};
use rand::seq::IndexedRandom;

#[cfg(target_arch = "wasm32")]
use crate::web::WebBgmAudioManager;

use crate::asset::{
    locale::{CurrentLocale, Locale},
    sound::SystemVolume,
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register systems to run when entering the `GameState::Option` state.
            .add_systems(
                OnEnter(GameState::Option),
                (debug_label, show_interface, init_slider_cursor_flag),
            )
            // Register a cleanup system to run when exiting the `GameState::Option` state.
            .add_systems(
                OnExit(GameState::Option),
                (hide_state_ui, clear_slider_cursor_flag),
            )
            .add_systems(
                PreUpdate,
                (
                    handle_player_input,
                    slider_interaction_system, // Update에서 PreUpdate로 이동
                )
                    .run_if(in_state(GameState::Option)),
            )
            // Register systems that run every frame while in the `GameState::Option` state.
            .add_systems(
                Update,
                (
                    update_slider_visual,
                    update_current_volume,
                    update_slider_cursor,
                    slider_feedback_system,
                    update_loacle_button,
                    update_back_button, // Note: This function handles the "Back" button.
                    control_background_volume,
                    control_effect_volume,
                    control_voice_volume,
                )
                    .run_if(in_state(GameState::Option)),
            );
    }
}

// --- RESOURCES ---

#[derive(Default, Resource)]
pub struct SelectedSliderCursor(Option<(UI, Entity)>);

impl SelectedSliderCursor {
    pub fn take(&mut self) -> Option<(UI, Entity)> {
        self.0.take()
    }

    pub fn get(&self) -> Option<(UI, Entity)> {
        self.0.clone()
    }

    pub fn set(&mut self, ui: UI, entity: Entity) {
        if self.0.is_none() {
            self.0 = Some((ui, entity))
        }
    }
}

// --- SETUP SYSTEMS ---

/// Prints a debug message to the console indicating the current game state.
fn debug_label() {
    info!("Current State: Option");
}

/// Makes all UI elements of the option screen visible.
fn show_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        // Match on UI components that belong to the options screen.
        match ui {
            UI::OptionModal
            | UI::SliderRail
            | UI::BgmLabel
            | UI::BgmVolume
            | UI::BgmVolumeCursor
            | UI::SfxLabel
            | UI::SfxVolume
            | UI::SfxVolumeCursor
            | UI::VoiceLabel
            | UI::VoiceVolume
            | UI::VoiceVolumeCursor
            | UI::BackButton
            | UI::LanguageEn
            | UI::LanguageJa
            | UI::LanguageKo => *visibility = Visibility::Visible,
            _ => { /* Do nothing for other UI elements. */ }
        }
    }
}

fn init_slider_cursor_flag(mut commands: Commands) {
    commands.insert_resource(SelectedSliderCursor::default());
}

// --- CLEANUP SYSTEMS ---

/// Hides all UI elements of the option screen.
fn hide_state_ui(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        // Match on UI components that belong to the options screen.
        match ui {
            UI::OptionModal
            | UI::SliderRail
            | UI::BgmLabel
            | UI::BgmVolume
            | UI::BgmVolumeCursor
            | UI::SfxLabel
            | UI::SfxVolume
            | UI::SfxVolumeCursor
            | UI::VoiceLabel
            | UI::VoiceVolume
            | UI::VoiceVolumeCursor
            | UI::BackButton
            | UI::LanguageEn
            | UI::LanguageJa
            | UI::LanguageKo => *visibility = Visibility::Hidden,
            _ => { /* Do nothing for other UI elements. */ }
        }
    }
}

fn clear_slider_cursor_flag(mut commands: Commands) {
    commands.remove_resource::<SelectedSliderCursor>();
}

// --- PREUPDATE SYSTEMS ---

fn handle_player_input(
    in_game_query: Query<(), With<InGameStateRoot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if in_game_query.is_empty() {
            next_state.set(GameState::Title);
        } else {
            next_state.set(GameState::Pause);
        }
    }
}

// --- UPDATE SYSTEMS ---

/// Provides visual feedback for volume slider handles based on their interaction state (hovered, pressed).
fn update_slider_visual(
    mut interaction_query: Query<(&UI, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (&ui, &interaction, mut color) in interaction_query.iter_mut() {
        // Darken the color of the slider handle based on interaction.
        match (ui, interaction) {
            (UI::BgmVolumeCursor, Interaction::Pressed)
            | (UI::SfxVolumeCursor, Interaction::Pressed)
            | (UI::VoiceVolumeCursor, Interaction::Pressed) => {
                *color = BackgroundColor(SLIDER_HANDLE_COLOR.darker(0.5));
            }
            (UI::BgmVolumeCursor, Interaction::Hovered)
            | (UI::SfxVolumeCursor, Interaction::Hovered)
            | (UI::VoiceVolumeCursor, Interaction::Hovered) => {
                *color = BackgroundColor(SLIDER_HANDLE_COLOR.darker(0.3));
            }
            (UI::BgmVolumeCursor, Interaction::None)
            | (UI::SfxVolumeCursor, Interaction::None)
            | (UI::VoiceVolumeCursor, Interaction::None) => {
                *color = BackgroundColor(SLIDER_HANDLE_COLOR);
            }
            _ => { /* empty */ }
        }
    }
}

/// Updates the volume percentage text displays (0-100) to match the values in the `SystemVolume` resource.
fn update_current_volume(system_volume: Res<SystemVolume>, mut query: Query<(&UI, &mut Text)>) {
    for (&ui, mut text) in query.iter_mut() {
        match ui {
            UI::BgmVolume => {
                *text = Text::new(format!(
                    "{}",
                    (100.0 * system_volume.background_percentage()).floor()
                ));
            }
            UI::SfxVolume => {
                *text = Text::new(format!(
                    "{}",
                    (100.0 * system_volume.effect_percentage()).floor()
                ));
            }
            UI::VoiceVolume => {
                *text = Text::new(format!(
                    "{}",
                    (100.0 * system_volume.voice_percentage()).floor()
                ));
            }
            _ => { /* empty */ }
        }
    }
}

fn slider_interaction_system(
    interaction_query: Query<(&UI, &Interaction, &ChildOf), Changed<Interaction>>,
    mut selected: ResMut<SelectedSliderCursor>,
) {
    for (&ui, &interaction, child_of) in interaction_query.iter() {
        match (ui, interaction) {
            (UI::BgmVolumeCursor, Interaction::Pressed) => {
                selected.set(UI::BgmVolumeCursor, child_of.parent());
            }
            (UI::SfxVolumeCursor, Interaction::Pressed) => {
                selected.set(UI::SfxVolumeCursor, child_of.parent());
            }
            (UI::VoiceVolumeCursor, Interaction::Pressed) => {
                selected.set(UI::VoiceVolumeCursor, child_of.parent());
            }
            _ => { /* empty */ }
        }
    }
}

fn update_slider_cursor(
    windows: Query<&Window>,
    mut node_query: Query<&mut Node>,
    mut system_volume: ResMut<SystemVolume>,
    selected: Res<SelectedSliderCursor>,
) {
    let Some((ui, entity)) = selected.get() else {
        return;
    };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let slider_width = window.width() * 0.5 * 0.4;
    let slider_begin = window.width() * 0.5 - slider_width / 2.0;
    let slider_end = window.width() * 0.5 + slider_width / 2.0;

    let slider_pos = cursor_position.x.clamp(slider_begin, slider_end);
    let percentage = (slider_pos - slider_begin) / slider_width;

    if let Ok(mut node) = node_query.get_mut(entity) {
        match ui {
            UI::BgmVolumeCursor => {
                node.left = Val::Percent(percentage * 100.0);
                system_volume.background = (percentage * 255.0).floor() as u8;
            }
            UI::SfxVolumeCursor => {
                node.left = Val::Percent(percentage * 100.0);
                system_volume.effect = (percentage * 255.0).floor() as u8;
            }
            UI::VoiceVolumeCursor => {
                node.left = Val::Percent(percentage * 100.0);
                system_volume.voice = (percentage * 255.0).floor() as u8;
            }
            _ => { /* empty */ }
        }
    }
}

fn slider_feedback_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut selected: ResMut<SelectedSliderCursor>,
) {
    if mouse_button.just_released(MouseButton::Left)
        && let Some((select, _)) = selected.take()
    {
        match select {
            UI::SfxVolumeCursor => {
                play_sfx_feedback_when_released(&mut commands, &asset_server, &system_volume);
            }
            UI::VoiceVolumeCursor => {
                play_voice_feedback_when_released(&mut commands, &asset_server, &system_volume);
            }
            _ => { /* empty */ }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_loacle_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mut current_locale: ResMut<CurrentLocale>,
    mut set: ParamSet<(
        Query<(&UI, &mut BackgroundColor), With<Button>>,
        Query<(&UI, &Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    )>,
) {
    for (&ui, &interaction, mut color) in set.p1().iter_mut() {
        match (ui, interaction, current_locale.0) {
            // Handle the 'English' button.
            (UI::LanguageEn, _, Locale::En) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
            } // Active state
            (UI::LanguageEn, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::LanguageEn, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                current_locale.0 = Locale::En;
            }
            (UI::LanguageEn, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            // Handle the 'Japanese' button.
            (UI::LanguageJa, _, Locale::Ja) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
            } // Active state
            (UI::LanguageJa, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::LanguageJa, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                current_locale.0 = Locale::Ja;
            }
            (UI::LanguageJa, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            // Handle the 'Korean' button.
            (UI::LanguageKo, _, Locale::Ko) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
            } // Active state
            (UI::LanguageKo, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::LanguageKo, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                current_locale.0 = Locale::Ko;
            }
            (UI::LanguageKo, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            _ => { /* empty */ }
        }
    }

    for (&ui, mut color) in set.p0().iter_mut() {
        match (ui, current_locale.0) {
            (UI::LanguageEn, Locale::En)
            | (UI::LanguageJa, Locale::Ja)
            | (UI::LanguageKo, Locale::Ko) => { /* empty */ }
            (UI::LanguageEn, _) | (UI::LanguageJa, _) | (UI::LanguageKo, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR)
            }
            _ => { /* empty */ }
        }
    }
}

/// Handles interactions with the 'Back' button.
/// It provides visual feedback and transitions back to the `Title` state when pressed.
#[allow(clippy::type_complexity)]
fn update_back_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    in_game_query: Query<(), With<InGameStateRoot>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&UI, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (&ui, &interaction, mut color) in interaction_query.iter_mut() {
        match (ui, interaction) {
            (UI::BackButton, Interaction::Hovered) => {
                *color = BackgroundColor(BACK_BTN_COLOR.darker(0.1));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::BackButton, Interaction::Pressed) => {
                *color = BackgroundColor(BACK_BTN_COLOR.darker(0.2));
                play_button_sound_when_returned(&mut commands, &asset_server, &system_volume);
                // Return to the previous screen.
                if in_game_query.is_empty() {
                    next_state.set(GameState::Title);
                } else {
                    next_state.set(GameState::Pause);
                }
            }
            (UI::BackButton, Interaction::None) => {
                *color = BackgroundColor(BACK_BTN_COLOR);
            }
            _ => { /* empty */ }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn control_background_volume(
    system_volume: Res<SystemVolume>,
    mut query: Query<&mut AudioSink, With<BackgroundSound>>,
) {
    if let Ok(mut sink) = query.single_mut() {
        sink.set_volume(Volume::Linear(system_volume.background_percentage()));
    }
}

#[cfg(target_arch = "wasm32")]
fn control_background_volume(
    system_volume: Res<SystemVolume>,
    web_bgm: NonSend<WebBgmAudioManager>,
) {
    web_bgm.set_volume(Volume::Linear(system_volume.background_percentage()));
}

fn control_effect_volume(
    system_volume: Res<SystemVolume>,
    mut query: Query<&mut AudioSink, With<EffectSound>>,
) {
    for mut sink in query.iter_mut() {
        sink.set_volume(Volume::Linear(system_volume.effect_percentage()));
    }
}

fn control_voice_volume(
    system_volume: Res<SystemVolume>,
    mut query: Query<&mut AudioSink, With<VoiceSound>>,
) {
    for mut sink in query.iter_mut() {
        sink.set_volume(Volume::Linear(system_volume.voice_percentage()));
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
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_TOUCH)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

fn play_button_sound_when_returned(
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

fn play_sfx_feedback_when_released(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_DOOR_BELL_00)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

fn play_voice_feedback_when_released(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    let path = SOUND_PATH_VO_TITLES
        .choose(&mut rand::rng())
        .copied()
        .unwrap();
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        VoiceSound,
    ));
}
