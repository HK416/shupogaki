use std::time::Duration;

// Import necessary Bevy modules.
use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, lens::UiPositionLens};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::StartInGame),
            (debug_label, play_ui_animation, show_in_game_interface),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label(mut next_state: ResMut<NextState<GameState>>) {
    info!("Current State: StartInGame");
    next_state.set(GameState::InGame);
}

fn show_in_game_interface(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, &ui) in query.iter_mut() {
        match ui {
            UI::StartLabel | UI::PauseButton | UI::Score | UI::Fuel => {
                *visibility = Visibility::Visible
            }
            _ => { /* empty */ }
        }
    }
}

fn play_ui_animation(mut commands: Commands, query: Query<(Entity, &UI)>) {
    for (entity, &ui) in query.iter() {
        match ui {
            UI::StartLabel => {
                commands
                    .entity(entity)
                    .insert(FadeInOutAnimation::new(PREPARE_ANIM_DURATION));
            }
            UI::PauseButton => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(PREPARE_ANIM_DURATION),
                    UiPositionLens {
                        start: UiRect {
                            left: Val::Auto,
                            right: Val::Vw(1.5),
                            top: Val::Vh(-20.0),
                            bottom: Val::Auto,
                        },
                        end: UiRect {
                            left: Val::Auto,
                            right: Val::Vw(1.5),
                            top: Val::Vh(1.5),
                            bottom: Val::Auto,
                        },
                    },
                )));
            }
            UI::Score => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(PREPARE_ANIM_DURATION),
                    UiPositionLens {
                        start: UiRect {
                            top: Val::Vh(-20.0),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                        end: UiRect {
                            top: Val::Vh(1.5),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                    },
                )));
            }
            UI::Fuel => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(PREPARE_ANIM_DURATION),
                    UiPositionLens {
                        start: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(-20.0),
                            right: Val::Vw(3.0),
                        },
                        end: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(1.5),
                            right: Val::Vw(3.0),
                        },
                    },
                )));
            }
            _ => { /* empty */ }
        }
    }
}
