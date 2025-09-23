// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};
use rand::seq::IndexedRandom;

use crate::{
    asset::{animation::AnimationClipHandle, material::EyeMouthMaterial, sound::SystemVolume},
    shader::face_mouth::EyeMouth,
};

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings};

use super::*;

// --- CONSTANTS ---

const SCENE_DURATION: f32 = 3.0;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::StartResult),
            (
                debug_label,
                start_timer,
                show_entities,
                spawn_camera_and_light,
                play_animation,
                play_result_sound,
                setup_result_text,
                check_and_save_high_score.after(setup_result_text),
            ),
        )
        .add_systems(OnExit(GameState::StartResult), end_timer)
        .add_systems(
            Update,
            (update_scene_timer, set_mouth_expression).run_if(in_state(GameState::StartResult)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: StartResult");
}

fn start_timer(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
}

fn show_entities(mut query: Query<&mut Visibility, (With<ResultStateRoot>, Without<UI>)>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
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
            Transform::from_xyz(8.0, 12.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ResultStateRoot,
        ));
    }

    if camera_query.single().is_err() {
        commands.spawn((
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                fov: 45f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
                near: 0.1,
                far: 100.0,
            }),
            Transform::from_translation(CAMERA_POSITION).looking_to(CAMERA_DIRECTION, Vec3::Y),
            ResultStateRoot,
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
        player.play(animation_index);

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player))
            .remove::<AnimationClipHandle>();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_result_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    let path = SOUND_PATH_VO_RESULTS
        .choose(&mut rand::rng())
        .copied()
        .unwrap();
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        VoiceSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_result_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    let path = SOUND_PATH_VO_RESULTS
        .choose(&mut rand::rng())
        .copied()
        .unwrap();
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(path)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        VoiceSound,
    ));
}

fn check_and_save_high_score(
    mut commands: Commands,
    mut high_score: ResMut<HighScore>,
    score: Res<CurrentScore>,
    new_record_query: Query<Entity, With<NewRecord>>,
) {
    if high_score.0 < score.get() {
        high_score.0 = score.get();

        if let Ok(entity) = new_record_query.single() {
            commands.entity(entity).insert(UI::NewRecord);
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(storage) = get_local_storage() {
            let _ = storage.set_item(HIGH_SCORE_KEY, &high_score.0.to_string());
        }
    }
}

fn setup_result_text(
    score: Res<CurrentScore>,
    play_time: Res<PlayTime>,
    high_score: Res<HighScore>,
    mut text_entities_query: Query<(&UI, &mut Text)>,
) {
    for (&ui, mut text) in text_entities_query.iter_mut() {
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
            UI::BestScore => {
                let high_score = score.get().max(high_score.0);
                *text = Text::new(high_score.to_string());
            }
            _ => { /* empty */ }
        }
    }
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
    if timer.elapsed_time >= SCENE_DURATION {
        next_state.set(GameState::Start2End);
    }
}

fn set_mouth_expression(
    mut materials: ResMut<Assets<EyeMouthMaterial>>,
    query: Query<&EyeMouth>,
    timer: ResMut<SceneTimer>,
) {
    for mouth in query.iter() {
        if let Some(material) = materials.get_mut(&mouth.0) {
            if timer.elapsed_time < SCENE_DURATION * 0.5 {
                material.extension.uniform.index.x = 2;
            } else {
                material.extension.uniform.index.x = 3;
            }
        }
    }
}
