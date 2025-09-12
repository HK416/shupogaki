// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::EndResult), debug_label)
            .add_systems(
                Update,
                (
                    fade_in_text,
                    fade_in_img_font,
                    fade_in_animation,
                    handle_button_system,
                )
                    .run_if(in_state(GameState::EndResult)),
            );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: EndResult");
}

// --- UPDATE SYSTEMS ---

fn fade_in_text(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TextColor, &mut FadeInAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut color, mut fade_out) in query.iter_mut() {
        fade_out.tick(time.delta_secs());
        if fade_out.is_expired() {
            color.0 = color.0.with_alpha(1.0);
            commands.entity(entity).remove::<FadeInAnimation>();
        } else {
            color.0 = color.0.with_alpha(fade_out.color().alpha());
        }
    }
}

fn fade_in_img_font(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImageNode, &mut FadeInAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut node, mut fade_out) in query.iter_mut() {
        fade_out.tick(time.delta_secs());
        if fade_out.is_expired() {
            node.color = node.color.with_alpha(1.0);
            commands.entity(entity).remove::<FadeInAnimation>();
        } else {
            node.color = node.color.with_alpha(fade_out.color().alpha());
        }
    }
}

#[allow(clippy::type_complexity)]
fn fade_in_animation(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut BackgroundColor, &mut FadeInAnimation),
        (Without<Text>, Without<ImageNode>),
    >,
    time: Res<Time>,
) {
    for (entity, mut color, mut fade_out) in query.iter_mut() {
        fade_out.tick(time.delta_secs());
        if fade_out.is_expired() {
            color.0 = color.0.with_alpha(1.0);
            commands.entity(entity).remove::<FadeInAnimation>();
        } else {
            color.0 = color.0.with_alpha(fade_out.color().alpha());
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
            (UI::RestartButton, Interaction::Hovered) => {
                color.0 = RESTART_BTN_COLOR.darker(0.15);
            }
            (UI::RestartButton, Interaction::Pressed) => {
                color.0 = RESTART_BTN_COLOR.darker(0.3);
                next_state.set(GameState::RestartResult);
            }
            (UI::RestartButton, Interaction::None) => {
                color.0 = RESUME_BTN_COLOR;
            }
            (UI::ResultExitButton, Interaction::Hovered) => {
                color.0 = EXIT_BTN_COLOR.darker(0.15);
            }
            (UI::ResultExitButton, Interaction::Pressed) => {
                color.0 = EXIT_BTN_COLOR.darker(0.3);
                next_state.set(GameState::ExitResult);
            }
            (UI::ResultExitButton, Interaction::None) => {
                color.0 = EXIT_BTN_COLOR;
            }
            _ => { /* empty */ }
        }
    }
}
