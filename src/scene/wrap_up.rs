// src/scene/wrap_up.rs

//! This module handles the `WrapUp` game state, which serves as the game-over
//! or level-complete sequence. It is responsible for:
//! - Animating the player off-screen.
//! - Animating the in-game UI (score, fuel) off-screen.
//! - Displaying a "Finish" message.
//! - Cleaning up all game-related entities and resources.
//! - Eventually transitioning to a score screen or main menu (currently a `TODO`).

use std::time::Duration;

// Import necessary Bevy modules.
use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, lens::UiPositionLens};

use super::*;

// --- CONSTANTS ---

/// The total duration of the wrap-up scene in seconds.
const SCENE_DURATION: f32 = 3.0;

/// The time it takes for the player to move off-screen during the wrap-up sequence.
const HIDE_DURATION: f32 = 1.5;

// --- SETUP SYSTEM ---

/// A system that runs once when entering `GameState::WrapUp`.
/// It makes the "Finish" UI visible and inserts the scene timer.
pub fn on_enter(mut commands: Commands, mut in_game_ui: Query<(&mut Visibility, &UI)>) {
    info!("Enter WrapUp State.");
    commands.insert_resource(SceneTimer::default());
    for (mut visibility, ui) in in_game_ui.iter_mut() {
        match *ui {
            // Make the "Finish" UI visible.
            UI::Finish => *visibility = Visibility::Visible,
            _ => { /* empty */ }
        }
    }
}

/// A system that starts the animations to slide the in-game UI elements off-screen.
pub fn play_ui_animation(mut commands: Commands, query: Query<(Entity, &UI)>) {
    for (entity, ui) in query.iter() {
        let mut commands = commands.entity(entity);
        match *ui {
            // Animate the score UI moving up and off-screen.
            UI::Score => {
                commands.insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(UI_ANIMATION_DURATION),
                    UiPositionLens {
                        start: UiRect {
                            top: Val::Vh(1.5),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                        end: UiRect {
                            top: Val::Vh(-20.0),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                    },
                )));
            }
            // Animate the fuel gauge UI moving down and off-screen.
            UI::Fuel => {
                commands.insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(UI_ANIMATION_DURATION),
                    UiPositionLens {
                        start: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(1.5),
                            right: Val::Vw(3.0),
                        },
                        end: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(-20.0),
                            right: Val::Vw(3.0),
                        },
                    },
                )));
            }
            _ => { /* empty */ }
        }
    }
}

// --- CLEANUP SYSTEM ---

/// A system that cleans up the game world when exiting the `WrapUp` state.
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<InGameStateEntity>>) {
    info!("Exit WrapUp State.");
    // Despawn all entities associated with the InGame state.
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove resources specific to the InGame state to prepare for a potential restart.
    commands.remove_resource::<RetiredGrounds>();
    commands.remove_resource::<CachedGrounds>();
    commands.remove_resource::<PlayerState>();
    commands.remove_resource::<PlayScore>();
}

// --- UPDATE SYSTEMS ---

/// A system that advances the scene timer and handles the end of the wrap-up sequence.
pub fn update_scene_timer(
    mut _next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.0 += time.delta_secs();
    if timer.0 >= SCENE_DURATION {
        // TODO: Implement the actual game over logic (e.g., transitioning to a score screen or main menu).
        todo!("Game Over!");
    }
}

/// A system that updates the alpha of the "Finish" UI element to fade it in.
pub fn update_finish_ui(
    mut query: Query<(&mut ImageNode, &mut FinishAnimation)>,
    timer: Res<SceneTimer>,
    time: Res<Time>,
) {
    for (mut image_node, mut finish_animation) in query.iter_mut() {
        // Start the fade-in animation partway through the scene.
        if timer.0 >= SCENE_DURATION - UI_ANIMATION_DURATION {
            finish_animation.update(time.delta_secs());
        }
        image_node.color = finish_animation.color();
    }
}

// --- POSTUPDATE SYSTEMS ---

/// A system that animates the player's position during the wrap-up sequence, moving them off-screen.
pub fn update_player_position(
    mut query: Query<&mut Transform, With<Player>>,
    timer: Res<SceneTimer>,
) {
    if let Ok(mut transform) = query.single_mut() {
        // Calculate the interpolation factor `t` from 0.0 to 1.0 based on the scene timer.
        let t = timer.0.min(HIDE_DURATION) / HIDE_DURATION;
        // Linearly interpolate the player's z-position from the gameplay position to a point off-screen.
        let z_pos = PLAYER_MAX_Z_POS * (1.0 - t) + PLAYER_MIN_Z_POS * t;
        transform.translation.z = z_pos;
    }
}

/// A system that gradually decelerates the player's forward speed to zero.
pub fn update_player_speed(mut query: Query<&mut ForwardMovement, With<Player>>, time: Res<Time>) {
    if let Ok(mut forward_move) = query.single_mut() {
        // Calculate the speed decrease for this frame. The deceleration is faster than the normal acceleration.
        let subtraction = 10.0 * ACCELERATION * time.delta_secs();
        // Subtract from the current velocity, ensuring it does not go below zero.
        forward_move.velocity = (forward_move.velocity - subtraction).max(0.0);
    }
}
