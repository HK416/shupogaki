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
            | UI::ResultExitButton
            | UI::BestScore
            | UI::NewRecord => *visibility = Visibility::Visible,
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
            | UI::ResultExitButton
            | UI::BestScore
            | UI::NewRecord => {
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
