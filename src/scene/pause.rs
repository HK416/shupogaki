// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Pause),
            (debug_label, show_interface, pause_animation),
        )
        .add_systems(OnExit(GameState::Pause), (hide_title, hide_interface))
        .add_systems(
            PreUpdate,
            handle_player_input.run_if(in_state(GameState::Pause)),
        )
        .add_systems(
            Update,
            (update_pause_title, handle_button_system).run_if(in_state(GameState::Pause)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Pause");
}

fn show_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::Pause => *visibility = Visibility::Visible,
            _ => { /* empty */ }
        }
    }
}

fn pause_animation(mut query: Query<&mut AnimationPlayer>) {
    for mut player in query.iter_mut() {
        player.pause_all();
    }
}

// --- CLEANUP SYSTEMS ---

fn hide_title(mut query: Query<&mut Visibility, With<PauseTitle>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Inherited;
    }
}

fn hide_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::Pause => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
    }
}

// --- PREUPDATE SYSTEMS ---

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Resume);
    }
}

// --- UPDATE SYSTEMS ---

fn update_pause_title(mut query: Query<&mut Visibility, With<PauseTitle>>, time: Res<Time>) {
    for mut visibility in query.iter_mut() {
        let t = time.elapsed_secs() % PAUSE_TITLE_CYCLE;
        if t < PAUSE_TITLE_CYCLE * 0.5 {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_button_system(
    mut query: Query<
        (&UI, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (ui, interaction, mut color) in query.iter_mut() {
        match (*ui, *interaction) {
            (UI::ResumeButton, Interaction::Hovered) => {
                color.0 = RESUME_BTN_COLOR.darker(0.15);
            }
            (UI::ResumeButton, Interaction::Pressed) => {
                color.0 = RESUME_BTN_COLOR.darker(0.3);
                next_state.set(GameState::Resume);
            }
            (UI::ResumeButton, Interaction::None) => {
                color.0 = RESUME_BTN_COLOR;
            }
            (UI::OptionButton, Interaction::Hovered) => {
                color.0 = OPTION_BTN_COLOR.darker(0.15);
            }
            (UI::OptionButton, Interaction::Pressed) => {
                color.0 = OPTION_BTN_COLOR.darker(0.3);
                next_state.set(GameState::Option);
            }
            (UI::OptionButton, Interaction::None) => {
                color.0 = OPTION_BTN_COLOR;
            }
            (UI::InGameExitButton, Interaction::Hovered) => {
                color.0 = EXIT_BTN_COLOR.darker(0.15);
            }
            (UI::InGameExitButton, Interaction::Pressed) => {
                color.0 = EXIT_BTN_COLOR.darker(0.3);
                next_state.set(GameState::ExitInGame);
            }
            (UI::InGameExitButton, Interaction::None) => {
                color.0 = EXIT_BTN_COLOR;
            }
            _ => { /* empty */ }
        }
    }
}
