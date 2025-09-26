// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};

use crate::asset::sound::SystemVolume;

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings};

use super::*;

// --- CONSTANTS ---
const SCENE_DURATION: f32 = 2.0;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::FinishedInGame),
            (
                debug_label,
                start_timer,
                show_in_game_interface,
                play_ui_animation,
                play_finish_sound,
            ),
        )
        .add_systems(OnExit(GameState::FinishedInGame), end_timer)
        .add_systems(
            Update,
            (
                update_scene_timer,
                update_ground_position,
                update_object_position,
                rotate_animation,
                fade_in_animation,
            )
                .run_if(in_state(GameState::FinishedInGame)),
        )
        .add_systems(
            PostUpdate,
            (spawn_grounds, update_player_speed).run_if(in_state(GameState::FinishedInGame)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: FinishedInGame");
}

fn start_timer(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
}

fn show_in_game_interface(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, &ui) in query.iter_mut() {
        match ui {
            UI::FinishLabel => *visibility = Visibility::Visible,
            _ => { /* empty */ }
        }
    }
}

fn play_ui_animation(mut commands: Commands, query: Query<(Entity, &UI)>) {
    for (entity, &ui) in query.iter() {
        match ui {
            UI::FinishLabel => {
                commands
                    .entity(entity)
                    .insert(FadeInAnimation::new(PREPARE_ANIM_DURATION));
            }
            _ => { /* empty */ }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_finish_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_FINISH)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_finish_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_FINISH)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

// --- CLEANUP SYSTEMS ---

fn end_timer(mut commands: Commands) {
    commands.remove_resource::<SceneTimer>();
}

// --- UPDATE SYSTEMS ---

fn update_scene_timer(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta_secs());
    if timer.elapsed_sec() >= SCENE_DURATION {
        next_state.set(GameState::CleanUpInGame);
    }
}

fn update_ground_position(
    player_query: Query<&ForwardMovement, With<Player>>,
    mut ground_entities: Query<(Entity, &mut Transform), With<Ground>>,
    mut retired: ResMut<RetiredGrounds>,
    time: Res<Time>,
) {
    let player_velocity = player_query
        .single()
        .map(|forward_move| forward_move.get())
        .unwrap_or(0.0);

    for (entity, mut transform) in ground_entities.iter_mut() {
        transform.translation.z -= player_velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_POSITION {
            retired.push(entity);
        }
    }
}

fn update_object_position(
    mut commands: Commands,
    mut object_entities: Query<(Entity, &mut Transform), With<Object>>,
    player_query: Query<&ForwardMovement, With<Player>>,
    time: Res<Time>,
) {
    let player_velocity = player_query
        .single()
        .map(|forward_move| forward_move.get())
        .unwrap_or(0.0);

    for (entity, mut transform) in object_entities.iter_mut() {
        transform.translation.z -= player_velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_POSITION {
            commands.entity(entity).despawn();
        }
    }
}

fn rotate_animation(mut query: Query<(&mut Transform, &RotateAnimation)>, time: Res<Time>) {
    for (mut transform, animation) in query.iter_mut() {
        let axis = animation.axis;
        let angle = animation.radian_per_sec * time.delta_secs();
        let rotation = Quat::from_axis_angle(axis, angle);
        transform.rotate(rotation);
    }
}

fn fade_in_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImageNode, &mut FadeInAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut image_node, mut fade_out) in query.iter_mut() {
        fade_out.tick(time.delta_secs());
        if fade_out.is_expired() {
            image_node.color = Color::WHITE;
            commands.entity(entity).remove::<FadeInAnimation>();
        } else {
            image_node.color = fade_out.color();
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

fn spawn_grounds(mut commands: Commands, mut retired: ResMut<RetiredGrounds>) {
    while let Some(entity) = retired.pop() {
        commands
            .entity(entity)
            .entry::<Transform>()
            .and_modify(|mut transform| {
                transform.translation.z += SPAWN_POSITION - DESPAWN_POSITION;
            })
            .or_insert(Transform::from_xyz(0.0, 0.0, SPAWN_POSITION));
    }
}

pub fn update_player_speed(
    mut player_query: Query<&mut ForwardMovement, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut forward_move) = player_query.single_mut() else {
        return;
    };

    let mut velocity = forward_move.get();
    velocity -= ACCELERATION * time.delta_secs();
    velocity = velocity.max(0.0);

    forward_move.set(velocity);
}
