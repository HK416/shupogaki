// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::animation::AnimationClipHandle;

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
            ),
        )
        .add_systems(OnExit(GameState::StartResult), end_timer)
        .add_systems(
            Update,
            update_scene_timer.run_if(in_state(GameState::StartResult)),
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
            Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
