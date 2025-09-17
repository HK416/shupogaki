// Import necessary Bevy modules.
use bevy::{audio::Volume, pbr::ExtendedMaterial, prelude::*};
use rand::seq::IndexedRandom;

use crate::{
    asset::{animation::AnimationClipHandle, sound::SystemVolume},
    shader::face_mouth::{EyeMouth, FacialExpressionExtension},
};

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
                fov: 50f32.to_radians(),
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
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FacialExpressionExtension>>>,
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
