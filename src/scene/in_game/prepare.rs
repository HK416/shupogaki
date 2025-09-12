// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use crate::asset::animation::AnimationClipHandle;

use super::*;

// --- CONSTANTS ---

const SCENE_DURATION: f32 = 3.0;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::PrepareInGame),
            (
                debug_label,
                start_timer,
                insert_resource,
                show_entities,
                spawn_camera_and_light,
                play_animation,
            ),
        )
        .add_systems(OnExit(GameState::PrepareInGame), end_timer)
        .add_systems(
            Update,
            (
                update_scene_timer,
                update_ground_position,
                update_object_position,
            )
                .run_if(in_state(GameState::PrepareInGame)),
        )
        .add_systems(
            PostUpdate,
            (
                update_player_position,
                update_toy_trains.after(update_player_position),
                spawn_grounds,
                spawn_objects,
            )
                .run_if(in_state(GameState::PrepareInGame)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: PrepareInGame");
}

fn start_timer(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
}

fn insert_resource(mut commands: Commands) {
    commands.insert_resource(Attacked::default());
    commands.insert_resource(PlayTime::default());
    commands.insert_resource(TrainFuel::default());
    commands.insert_resource(InputDelay::default());
    commands.insert_resource(CurrentLane::default());
    commands.insert_resource(CurrentScore::default());
    commands.insert_resource(ForwardMovement::default());
    commands.insert_resource(VerticalMovement::default());
    commands.insert_resource(CurrentState::default());
    commands.insert_resource(RetiredGrounds::default());
    commands.insert_resource(ObjectSpawner::default());
}

fn show_entities(mut query: Query<&mut Visibility, (With<InGameStateRoot>, Without<UI>)>) {
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
            InGameStateRoot,
        ));
    }

    if camera_query.single().is_err() {
        commands.spawn((
            Camera3d::default(),
            Projection::from(OrthographicProjection {
                near: 0.1,
                far: 100.0,
                scaling_mode: ScalingMode::Fixed {
                    width: 16.0,
                    height: 9.0,
                },
                scale: 1.25,
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_xyz(12.0, 9.0, 12.0).looking_at((0.0, 1.5, 0.0).into(), Vec3::Y),
            InGameStateRoot,
        ));
    }

    commands.insert_resource(ClearColor(CLEAR_COLOR));
}

fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<(Entity, &AnimationClipHandle), With<InGameStateEntity>>,
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
        next_state.set(GameState::StartInGame);
    }
}

fn update_ground_position(
    mut commands: Commands,
    current: Res<ForwardMovement>,
    mut query: Query<(Entity, &mut Transform), With<Ground>>,
    mut retired: ResMut<RetiredGrounds>,
    time: Res<Time>,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation.z -= current.velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_LOCATION {
            retired.push(*transform);
            commands.entity(entity).despawn();
        }
    }
}

fn update_object_position(
    mut commands: Commands,
    current: Res<ForwardMovement>,
    mut query: Query<(Entity, &mut Transform), With<Object>>,
    time: Res<Time>,
) {
    for (entity, mut transform) in query.iter_mut() {
        transform.translation.z -= current.velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_LOCATION {
            commands.entity(entity).despawn();
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

fn update_player_position(mut query: Query<&mut Transform, With<Player>>, timer: Res<SceneTimer>) {
    if let Ok(mut transform) = query.single_mut() {
        // Calculate the interpolation factor `t` from 0.0 to 1.0 based on the scene timer.
        let t = timer.elapsed_time / SCENE_DURATION;
        // Linearly interpolate the player's z-position from the starting point to the final gameplay position.
        let z_pos = PLAYER_MIN_Z_POS * (1.0 - t) + PLAYER_MAX_Z_POS * t;
        transform.translation.z = z_pos;
    }
}

#[allow(clippy::type_complexity)]
fn update_toy_trains(
    mut set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<ToyTrain0>>,
        Query<&mut Transform, With<ToyTrain1>>,
        Query<&mut Transform, With<ToyTrain2>>,
    )>,
) {
    let data = set
        .p0()
        .single()
        .map(|transform| transform.translation.with_z(transform.translation.z + 1.5))
        .ok();

    if let Some(p_pos) = data {
        let mut position = p_pos;

        if let Ok(mut transform) = set.p1().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            let temp = transform.translation;

            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = p_pos.z - 1.5;
            transform.rotation = rotation;

            position = temp;
        }

        if let Ok(mut transform) = set.p2().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            let temp = transform.translation;
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = p_pos.z - 3.0;
            transform.rotation = rotation;

            position = temp;
        }

        if let Ok(mut transform) = set.p3().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = p_pos.z - 4.25;
            transform.rotation = rotation;
        }
    }
}

fn spawn_grounds(
    mut commands: Commands,
    mut retired: ResMut<RetiredGrounds>,
    asset_server: Res<AssetServer>,
) {
    while let Some(mut transform) = retired.pop() {
        transform.translation.z += SPAWN_LOCATION - DESPAWN_LOCATION;
        let model = asset_server.load(MODEL_PATH_PLANE_0);
        commands.spawn((SpawnModel(model), transform, InGameStateRoot, Ground));
    }
}

fn spawn_objects(
    mut commands: Commands,
    mut spawner: ResMut<ObjectSpawner>,
    asset_server: Res<AssetServer>,
    current: Res<ForwardMovement>,
    time: Res<Time>,
) {
    spawner.on_advanced(&mut commands, &asset_server, &current, time.delta_secs());
}
