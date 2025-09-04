// src/scene/resume.rs

//! This module handles the `Resume` game state, which displays a countdown
//! before returning to `InGame` from the `Pause` state.

// Import necessary Bevy modules.
use bevy::prelude::*;

// Import common game scene components and constants.
use super::*;

// --- CONSTANTS ---
/// The total duration of the resume countdown scene in seconds.
const SCENE_DURATION: f32 = 3.0;

// --- SETUP SYSTEM ---

/// A system that runs once when entering `GameState::Resume`.
/// It initializes and inserts the `SceneTimer` resource to track the countdown duration.
pub fn on_enter(mut commands: Commands) {
    info!("Enter Resume state.");
    commands.insert_resource(SceneTimer::default());
}

// --- CLEANUP SYSTEM ---

/// A system that runs once when exiting `GameState::Resume`.
/// It removes the `SceneTimer` resource to clean up state-specific data.
pub fn on_exit(mut commands: Commands) {
    info!("Exit Resume state.");
    commands.remove_resource::<SceneTimer>();
}

/// A system that resumes all previously paused animations when exiting the `GameState::Resume`.
pub fn resume_animation(mut query: Query<&mut AnimationPlayer>) {
    for mut player in query.iter_mut() {
        player.resume_all();
    }
}

// --- UPDATE SYSTEMS ---

/// A system that advances the scene timer and transitions the game state.
/// When the `SCENE_DURATION` is reached, it sets the next state to `GameState::InGame`
/// to resume gameplay.
pub fn update_scene_timer(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta_secs());
    // If the timer has reached or exceeded the scene duration, transition back to InGame.
    if timer.0 >= SCENE_DURATION {
        next_state.set(GameState::InGame);
    }
}

/// A system that updates the visibility of the resume countdown UI elements (3, 2, 1).
/// It makes each number visible sequentially based on the `SceneTimer`'s progress,
/// creating a countdown effect.
pub fn update_resume_ui(mut query: Query<(&mut Visibility, &UI)>, timer: Res<SceneTimer>) {
    for (mut visibility, ui) in query.iter_mut() {
        match (*ui, timer.0) {
            // For "ResumeCount1" (the number 1):
            // Hidden during the first two seconds, then visible for the last second.
            (UI::ResumeCount1, 0.0..1.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount1, 1.0..2.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount1, 2.0..) => {
                // Visible from 2.0 seconds onwards.
                *visibility = Visibility::Visible;
            }
            // For "ResumeCount2" (the number 2):
            // Hidden during the first second, visible during the second second, then hidden again.
            (UI::ResumeCount2, 0.0..1.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount2, 1.0..2.0) => {
                *visibility = Visibility::Visible;
            }
            (UI::ResumeCount2, 2.0..) => {
                *visibility = Visibility::Hidden;
            } // Hidden from 2.0 seconds onwards.
            // For "ResumeCount3" (the number 3):
            // Visible during the first second, then hidden for the rest of the countdown.
            (UI::ResumeCount3, 0.0..1.0) => {
                *visibility = Visibility::Visible;
            }
            (UI::ResumeCount3, 1.0..2.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount3, 2.0..) => {
                // Hidden from 2.0 seconds onwards.
                *visibility = Visibility::Hidden;
            }
            _ => { /* empty */ }
        }
    }
}
