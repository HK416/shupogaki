// src/scene/pause.rs

//! This module contains all the systems and components specific to the `Pause`
//! game state. This includes enabling the pause UI, pausing animations, and
//! handling input to resume the game.

// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- SETUP SYSTEM ---

/// A system that runs once when entering `GameState::Pause`.
/// It makes the "Pause" UI visible.
pub fn enable_pause_ui(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, ui) in query.iter_mut() {
        match *ui {
            // Make the pause title, resume button, and exit button visible.
            UI::PauseTitle | UI::ResumeButton | UI::ExitButton => *visibility = Visibility::Visible,
            // Ignore all other UI elements, as they are not part of the pause screen.
            _ => { /* empty */ }
        }
    }
}

/// A system that pauses all active animations when entering the `GameState::Pause`.
pub fn pause_animation(mut query: Query<&mut AnimationPlayer>) {
    for mut player in query.iter_mut() {
        player.pause_all();
    }
}

// --- CLEANUP SYSTEM ---

/// A system that runs once when exiting `GameState::Pause`.
/// It hides the "Pause" UI.
pub fn disable_pause_ui(mut query: Query<(&mut Visibility, &UI)>) {
    info!("Exit Pause state.");
    for (mut visibility, ui) in query.iter_mut() {
        match *ui {
            // Hide the pause title, resume button, and exit button.
            UI::PauseTitle | UI::ResumeButton | UI::ExitButton => *visibility = Visibility::Hidden,
            // Ignore all other UI elements.
            _ => { /* empty */ }
        }
    }
}

/// A system that hides the pause title when exiting the `GameState::Pause`.
/// NOTE: This system's functionality is also handled by `disable_pause_ui`.
/// It might be redundant and could potentially be removed or merged in the future
/// unless a more specific ordering or logic is required for the title.
pub fn disable_pause_title(mut query: Query<&mut Visibility, With<PauseTitle>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

// --- PREUPDATE SYSTEMS ---

/// A system that handles player input while in the `GameState::Pause`.
/// Currently, it only checks for the Escape key, but does not change the state here.
/// The state change logic for unpausing is handled in `in_game.rs`.
pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Resume);
    }
}

// --- UPDATE SYSTEMS ---

/// A system that makes the pause title blink by toggling its visibility.
pub fn update_pause_title(mut query: Query<&mut Visibility, With<PauseTitle>>, time: Res<Time>) {
    for mut visibility in query.iter_mut() {
        // Use the modulo operator to create a cyclical timer.
        // The title will be visible for the first half of the cycle and hidden for the second half.
        let t = time.elapsed_secs() % PAUSE_TITLE_CYCLE;
        if t < PAUSE_TITLE_CYCLE * 0.5 {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// A system that handles interactions with the "Resume" and "Exit" buttons in the pause menu.
#[allow(clippy::type_complexity)]
pub fn button_system(
    mut query: Query<
        (&UI, &Interaction, &mut BackgroundColor, &Children),
        // This query only runs when a button's interaction state changes (e.g., hovered, pressed).
        // This is a Bevy performance optimization to prevent the system from running on every frame.
        (Changed<Interaction>, With<Button>),
    >,
    mut nodes: Query<&mut ImageNode>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (ui, interaction, mut color, children) in query.iter_mut() {
        match (*ui, *interaction) {
            (UI::ResumeButton, Interaction::Hovered) => {
                // Slightly darken the button on hover for visual feedback.
                color.0 = RESUME_BTN_COLOR.darker(0.15);
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(0.85, 0.85, 0.85);
                    }
                }
            }
            (UI::ResumeButton, Interaction::Pressed) => {
                // Darken it more when pressed.
                color.0 = RESUME_BTN_COLOR.darker(0.3);
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(0.7, 0.7, 0.7);
                    }
                }
                // Transition to the Resume state to unpause the game.
                next_state.set(GameState::Resume);
            }
            (UI::ResumeButton, Interaction::None) => {
                // Return to the original color when not interacting.
                color.0 = RESUME_BTN_COLOR;
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(1.0, 1.0, 1.0);
                    }
                }
            }
            (UI::ExitButton, Interaction::Hovered) => {
                color.0 = EXIT_BTN_COLOR.darker(0.15);
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(0.85, 0.85, 0.85);
                    }
                }
            }
            (UI::ExitButton, Interaction::Pressed) => {
                color.0 = EXIT_BTN_COLOR.darker(0.3);
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(0.7, 0.7, 0.7);
                    }
                }
                // TODO: Implement application exit logic.
                // This should gracefully close the game window.
                todo!("Game exit!");
            }
            (UI::ExitButton, Interaction::None) => {
                color.0 = EXIT_BTN_COLOR;
                for child in children.iter() {
                    if let Ok(mut image) = nodes.get_mut(child) {
                        image.color = Color::srgb(1.0, 1.0, 1.0);
                    }
                }
            }
            // Ignore other UI elements that might have a Button component.
            _ => { /* empty */ }
        }
    }
}
