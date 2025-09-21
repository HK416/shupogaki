// Import necessary Bevy modules.
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::web::WebPlaybackSettings;

use super::*;

// --- CONSTANTS ---
const SCENE_DURATION: f32 = 3.0;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Resume), (debug_label, start_timer))
            .add_systems(
                OnExit(GameState::Resume),
                (
                    end_timer,
                    hide_interface,
                    resume_animation,
                    resume_effect_sounds,
                    resume_voice_sounds,
                ),
            )
            .add_systems(
                Update,
                (update_scene_timer, update_resume_ui).run_if(in_state(GameState::Resume)),
            );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Resume");
}

fn start_timer(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
}

// --- CLEANUP SYSTEMS ---

fn end_timer(mut commands: Commands) {
    commands.remove_resource::<SceneTimer>();
}

fn hide_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::ResumeCount1 | UI::ResumeCount2 | UI::ResumeCount3 => {
                *visibility = Visibility::Hidden
            }
            _ => { /* empty */ }
        }
    }
}

fn resume_animation(mut query: Query<&mut AnimationPlayer>) {
    for mut player in query.iter_mut() {
        player.resume_all();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resume_effect_sounds(
    mut query: Query<&mut AudioSink, (With<InGameStateRoot>, With<EffectSound>)>,
) {
    for sink in query.iter_mut() {
        sink.play();
    }
}

#[cfg(target_arch = "wasm32")]
fn resume_effect_sounds(
    mut query: Query<&mut WebPlaybackSettings, (With<InGameStateRoot>, With<EffectSound>)>,
) {
    for mut settings in query.iter_mut() {
        settings.paused = false;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resume_voice_sounds(
    mut query: Query<&mut AudioSink, (With<InGameStateRoot>, With<VoiceSound>)>,
) {
    for sink in query.iter_mut() {
        sink.play();
    }
}

#[cfg(target_arch = "wasm32")]
fn resume_voice_sounds(
    mut query: Query<&mut WebPlaybackSettings, (With<InGameStateRoot>, With<VoiceSound>)>,
) {
    for mut settings in query.iter_mut() {
        settings.paused = false;
    }
}

// --- UPDATE SYSTEMS ---

fn update_scene_timer(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta_secs());
    // If the timer has reached or exceeded the scene duration, transition back to InGame.
    if timer.elapsed_time >= SCENE_DURATION {
        next_state.set(GameState::InGame);
    }
}

fn update_resume_ui(mut query: Query<(&UI, &mut Visibility)>, timer: Res<SceneTimer>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match (ui, timer.elapsed_time) {
            (UI::ResumeCount1, 0.0..1.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount1, 1.0..2.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount1, 2.0..) => {
                *visibility = Visibility::Visible;
            }
            (UI::ResumeCount2, 0.0..1.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount2, 1.0..2.0) => {
                *visibility = Visibility::Visible;
            }
            (UI::ResumeCount2, 2.0..) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount3, 0.0..1.0) => {
                *visibility = Visibility::Visible;
            }
            (UI::ResumeCount3, 1.0..2.0) => {
                *visibility = Visibility::Hidden;
            }
            (UI::ResumeCount3, 2.0..) => {
                *visibility = Visibility::Hidden;
            }
            _ => { /* empty */ }
        }
    }
}
