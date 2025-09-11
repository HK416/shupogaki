// Import necessary Bevy modules.
use bevy::prelude::*;

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
            .add_systems(OnEnter(GameState::Option), (debug_label, show_interface))
            // Register a cleanup system to run when exiting the `GameState::Option` state.
            .add_systems(OnExit(GameState::Option), hide_state_ui)
            .add_systems(
                PreUpdate,
                handle_player_input.run_if(in_state(GameState::Option)),
            )
            // Register systems that run every frame while in the `GameState::Option` state.
            .add_systems(
                Update,
                (
                    update_slider_visual,
                    slider_interaction_system,
                    update_current_volume,
                    update_current_locale,
                    update_okay_button, // Note: This function handles the "Back" button.
                )
                    .run_if(in_state(GameState::Option)),
            );
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

/// Handles the logic for dragging a volume slider handle.
/// It calculates the new volume percentage based on cursor position, updates the handle's visual position,
/// and modifies the `SystemVolume` resource.
fn slider_interaction_system(
    windows: Query<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut system_volume: ResMut<SystemVolume>,
    mut interaction_query: Query<(&UI, &Interaction, &ChildOf)>,
    mut node_query: Query<&mut Node>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    for (&ui, &interaction, child_of) in interaction_query.iter_mut() {
        // Check if a slider handle is being pressed.
        if interaction == Interaction::Pressed && mouse_button.pressed(MouseButton::Left) {
            // Calculate the slider's dimensions and position.
            let slider_width = window.width() * 0.5 * 0.4;
            let slider_begin = window.width() * 0.5 - slider_width / 2.0;
            let slider_end = window.width() * 0.5 + slider_width / 2.0;

            // Clamp the cursor position to the slider's bounds.
            let slider_pos = cursor_position.x.clamp(slider_begin, slider_end);
            let percentage = (slider_pos - slider_begin) / slider_width;

            // Update the slider handle's node position and the corresponding volume resource.
            if let Ok(mut node) = node_query.get_mut(child_of.parent()) {
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
            };
        }
    }
}

/// Manages the language selection buttons.
/// It updates their visual state (hover, pressed, active) and changes the `CurrentLocale` resource upon selection.
fn update_current_locale(
    mut current_locale: ResMut<CurrentLocale>,
    mut interaction_query: Query<(&UI, &Interaction, &mut BackgroundColor)>,
) {
    for (&ui, &interaction, mut color) in interaction_query.iter_mut() {
        match (ui, interaction, current_locale.0) {
            // Handle the 'English' button.
            (UI::LanguageEn, _, Locale::En) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5))
            } // Active state
            (UI::LanguageEn, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3))
            }
            (UI::LanguageEn, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                current_locale.0 = Locale::En;
            }
            (UI::LanguageEn, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            // Handle the 'Japanese' button.
            (UI::LanguageJa, _, Locale::Ja) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5))
            } // Active state
            (UI::LanguageJa, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3))
            }
            (UI::LanguageJa, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                current_locale.0 = Locale::Ja;
            }
            (UI::LanguageJa, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            // Handle the 'Korean' button.
            (UI::LanguageKo, _, Locale::Ko) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5))
            } // Active state
            (UI::LanguageKo, Interaction::Hovered, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.3))
            }
            (UI::LanguageKo, Interaction::Pressed, _) => {
                *color = BackgroundColor(LANGUAGE_BTN_COLOR.darker(0.5));
                current_locale.0 = Locale::Ko;
            }
            (UI::LanguageKo, Interaction::None, _) => *color = BackgroundColor(LANGUAGE_BTN_COLOR),

            _ => { /* empty */ }
        }
    }
}

/// Handles interactions with the 'Back' button.
/// It provides visual feedback and transitions back to the `Title` state when pressed.
fn update_okay_button(
    in_game_query: Query<(), With<InGameStateRoot>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&UI, &Interaction, &mut BackgroundColor)>,
) {
    for (&ui, &interaction, mut color) in interaction_query.iter_mut() {
        match (ui, interaction) {
            (UI::BackButton, Interaction::Hovered) => {
                *color = BackgroundColor(EXIT_BTN_COLOR.darker(0.1));
            }
            (UI::BackButton, Interaction::Pressed) => {
                *color = BackgroundColor(EXIT_BTN_COLOR.darker(0.2));
                // Return to the previous screen.
                if in_game_query.is_empty() {
                    next_state.set(GameState::Title);
                } else {
                    next_state.set(GameState::Pause);
                }
            }
            (UI::BackButton, Interaction::None) => {
                *color = BackgroundColor(EXIT_BTN_COLOR);
            }
            _ => { /* empty */ }
        }
    }
}
