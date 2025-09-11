// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::animation::AnimationClipHandle;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Title),
            (
                debug_label,
                show_entities,
                show_interfaces,
                spawn_camera_and_light,
                play_animation,
            ),
        )
        .add_systems(OnExit(GameState::Title), hide_interfaces)
        .add_systems(
            Update,
            title_button_systems.run_if(in_state(GameState::Title)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Title");
}

fn show_entities(mut query: Query<&mut Visibility, (With<TitleStateRoot>, Without<UI>)>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

fn show_interfaces(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::StartButton | UI::OptionButton | UI::RankButton => {
                *visibility = Visibility::Visible
            }
            _ => { /* empty */ }
        }
    }
}

fn spawn_camera_and_light(
    mut commands: Commands,
    light_query: Query<(), With<DirectionalLight>>,
    camera_query: Query<(), With<Camera3d>>,
) {
    if light_query.single().is_err() {
        commands.spawn((
            DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            Transform::from_xyz(8.0, 12.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            TitleStateRoot,
        ));
    }

    if camera_query.single().is_err() {
        commands.spawn((
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                fov: 50f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
                near: 0.1,
                far: 100.0,
            }),
            Transform::from_translation(CAMERA_POSITION).looking_to(CAMERA_DIRECTION, Vec3::Y),
            TitleStateRoot,
        ));
    }

    commands.insert_resource(ClearColor(CLEAR_COLOR));
}

fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<(Entity, &AnimationClipHandle)>,
) {
    for (entity, clip) in query.iter() {
        let (graph, animation_index) = AnimationGraph::from_clip(clip.0.clone());
        let mut player = AnimationPlayer::default();
        player.play(animation_index).repeat();

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player))
            .remove::<AnimationClipHandle>();
    }
}

// --- CLEANUP SYSTEM ---

fn hide_interfaces(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::StartButton | UI::OptionButton | UI::RankButton => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
    }
}

// --- UPDATE SYSTEM ---

#[allow(clippy::type_complexity)]
fn title_button_systems(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&UI, &Interaction, &mut TextColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (&ui, interaction, mut text_color) in interaction_query.iter_mut() {
        match (ui, interaction) {
            (UI::StartButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
            }
            (UI::StartButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
                next_state.set(GameState::Title2InGame);
            }
            (UI::StartButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            (UI::OptionButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
            }
            (UI::OptionButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
                next_state.set(GameState::Option);
            }
            (UI::OptionButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            (UI::RankButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
            }
            (UI::RankButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
            }
            (UI::RankButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            _ => { /* empty */ }
        }
    }
}
