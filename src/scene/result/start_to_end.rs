// Import necessary Bevy modules.
use bevy::prelude::*;

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Start2End),
            (
                debug_label,
                show_interface,
                set_result_text,
                play_ui_animation,
                play_hikari_animation,
                play_nozomi_animation,
            ),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label(mut next_state: ResMut<NextState<GameState>>) {
    info!("Current State: Start2End");
    next_state.set(GameState::EndResult);
}

fn show_interface(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::ResultText
            | UI::ResultImgFont
            | UI::PlayTime
            | UI::GameScore
            | UI::ResultModal
            | UI::RestartButton
            | UI::ResultExitButton => *visibility = Visibility::Visible,
            _ => { /* empty */ }
        }
    }
}

fn set_result_text(
    score: Res<CurrentScore>,
    play_time: Res<PlayTime>,
    mut query: Query<(&UI, &mut Text)>,
) {
    for (&ui, mut text) in query.iter_mut() {
        match ui {
            UI::PlayTime => {
                let total_millis = play_time.millis();
                let minutes = (total_millis / (1000 * 60)) % 60;
                let seconds = (total_millis / 1000) % 60;
                let milliseconds = total_millis % 1000;
                *text = Text::new(format!("{:02}:{:02}:{:03}", minutes, seconds, milliseconds));
            }
            UI::GameScore => {
                *text = Text::new(score.get().to_string());
            }
            _ => { /* empty */ }
        }
    }
}

fn play_ui_animation(mut commands: Commands, query: Query<(Entity, &UI)>) {
    for (entity, &ui) in query.iter() {
        match ui {
            UI::ResultText
            | UI::ResultImgFont
            | UI::PlayTime
            | UI::GameScore
            | UI::ResultModal
            | UI::RestartButton
            | UI::ResultExitButton => {
                commands
                    .entity(entity)
                    .insert(FadeInAnimation::new(PREPARE_ANIM_DURATION));
            }
            _ => { /* empty */ }
        }
    }
}

fn play_hikari_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<Entity, (With<ResultStateEntity>, With<Hikari>)>,
) {
    let clip = asset_server.load(ANIM_PATH_HIKARI_VICTORY_END);
    for entity in query.iter() {
        let (graph, animation_index) = AnimationGraph::from_clip(clip.clone());
        let mut player = AnimationPlayer::default();
        player.play(animation_index).repeat();

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player));
    }
}

fn play_nozomi_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<Entity, (With<ResultStateEntity>, With<Nozomi>)>,
) {
    let clip = asset_server.load(ANIM_PATH_NOZOMI_VICTORY_END);
    for entity in query.iter() {
        let (graph, animation_index) = AnimationGraph::from_clip(clip.clone());
        let mut player = AnimationPlayer::default();
        player.play(animation_index).repeat();

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player));
    }
}
