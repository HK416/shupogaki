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
            // Make the pause title visible.
            UI::PauseTitle => *visibility = Visibility::Visible,
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
            // Hide the pause title.
            UI::PauseTitle => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
    }
}

/// A system that hides the pause title when exiting the `GameState::Pause`.
/// This is a separate system from `disable_pause_ui` to specifically target the `PauseTitle` component.
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
        let t = time.elapsed_secs() % 2.0;
        if t < 1.0 {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
