use std::time::Duration;

// Import necessary Bevy modules.
use bevy::prelude::*;
use bevy_tweening::{Animator, Tween, TweenCompleted, lens::UiPositionLens};

use super::*;

// --- CONSTANTS ---
const SCENE_DURATION: f32 = 2.5;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::WrapUpInGame),
            (debug_label, start_timer, play_ui_animation),
        )
        .add_systems(
            OnExit(GameState::WrapUpInGame),
            (end_timer, hide_in_game_interface),
        )
        .add_systems(
            Update,
            (
                update_scene_timer,
                update_player_state,
                update_ground_position,
                update_object_position,
                rotate_animation,
                cleanup_ui_animation,
            )
                .run_if(in_state(GameState::WrapUpInGame)),
        )
        .add_systems(
            PreUpdate,
            (
                update_player_position,
                update_toy_trains,
                spawn_grounds,
                update_player_effect,
            )
                .run_if(in_state(GameState::WrapUpInGame)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: WrapupInGame");
}

fn start_timer(mut commands: Commands) {
    commands.insert_resource(SceneTimer::default());
}

fn play_ui_animation(mut commands: Commands, query: Query<(Entity, &UI)>) {
    for (entity, &ui) in query.iter() {
        match ui {
            UI::PauseButton => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(FINISH_ANIM_DURATION),
                    UiPositionLens {
                        end: UiRect {
                            left: Val::Auto,
                            right: Val::Vw(1.5),
                            top: Val::Vh(-20.0),
                            bottom: Val::Auto,
                        },
                        start: UiRect {
                            left: Val::Auto,
                            right: Val::Vw(1.5),
                            top: Val::Vh(1.5),
                            bottom: Val::Auto,
                        },
                    },
                )));
            }
            UI::Score => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(FINISH_ANIM_DURATION),
                    UiPositionLens {
                        end: UiRect {
                            top: Val::Vh(-20.0),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                        start: UiRect {
                            top: Val::Vh(1.5),
                            left: Val::Vw(1.5),
                            bottom: Val::Auto,
                            right: Val::Auto,
                        },
                    },
                )));
            }
            UI::Fuel => {
                commands.entity(entity).insert(Animator::new(Tween::new(
                    EaseFunction::SmoothStep,
                    Duration::from_secs_f32(FINISH_ANIM_DURATION),
                    UiPositionLens {
                        end: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(-20.0),
                            right: Val::Vw(3.0),
                        },
                        start: UiRect {
                            top: Val::Auto,
                            left: Val::Auto,
                            bottom: Val::Vh(1.5),
                            right: Val::Vw(3.0),
                        },
                    },
                )));
            }
            _ => { /* empty */ }
        }
    }
}

// --- CLEANUP SYSTEMS ---

fn end_timer(mut commands: Commands) {
    commands.remove_resource::<SceneTimer>();
}

fn hide_in_game_interface(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, &ui) in query.iter_mut() {
        match ui {
            UI::StartLabel | UI::PauseButton | UI::Score | UI::Fuel => {
                *visibility = Visibility::Hidden
            }
            _ => { /* empty */ }
        }
    }
}

// --- UPDATE SYSTEMS ---

fn update_scene_timer(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SceneTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta_secs());
    if timer.elapsed_time >= SCENE_DURATION {
        next_state.set(GameState::FinishedInGame);
    }
}

fn update_player_state(mut state: ResMut<CurrentState>, time: Res<Time>) {
    match &mut *state {
        CurrentState::Attacked { remaining } => {
            *remaining -= time.delta_secs();
            if *remaining <= 0.0 {
                *state = CurrentState::Idle;
            }
        }
        _ => { /* empty */ }
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

fn rotate_animation(mut query: Query<(&mut Transform, &RotateAnimation)>, time: Res<Time>) {
    for (mut transform, animation) in query.iter_mut() {
        let axis = animation.axis;
        let angle = animation.radian_per_sec * time.delta_secs();
        let rotation = Quat::from_axis_angle(axis, angle);
        transform.rotate(rotation);
    }
}

fn cleanup_ui_animation(
    mut commands: Commands,
    mut reader: EventReader<TweenCompleted>,
    query: Query<(), With<Animator<Node>>>,
) {
    for event in reader.read() {
        let entity = event.entity;
        if query.contains(entity) {
            commands.entity(entity).remove::<Animator<Node>>();
            info!("Animator removed!");
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

fn update_player_position(mut query: Query<&mut Transform, With<Player>>, timer: Res<SceneTimer>) {
    if let Ok(mut transform) = query.single_mut() {
        // Calculate the interpolation factor `t` from 0.0 to 1.0 based on the scene timer.
        let t = timer.elapsed_time / SCENE_DURATION;
        // Linearly interpolate the player's z-position from the starting point to the final gameplay position.
        let z_pos = PLAYER_MAX_Z_POS * (1.0 - t) + PLAYER_MIN_Z_POS * t;
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

#[allow(clippy::type_complexity)]
pub fn update_player_effect(
    mut set: ParamSet<(
        Query<Entity, With<ToyTrain0>>,
        Query<Entity, With<ToyTrain1>>,
        Query<Entity, With<ToyTrain2>>,
    )>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<CurrentState>,
) {
    if let Ok(entity) = set.p0().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &mut state,
        );
    }

    if let Ok(entity) = set.p1().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &mut state,
        );
    }

    if let Ok(entity) = set.p2().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &mut state,
        );
    }
}

fn update_player_effect_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut CurrentState,
) {
    if let Ok(handle) = material_query.get(entity)
        && let Some(material) = materials.get_mut(handle.id())
    {
        match &mut *state {
            #[cfg(not(feature = "no-debuging-player"))]
            CurrentState::Debug => {
                material.base_color = Color::BLACK;
            }
            CurrentState::Idle => {
                material.base_color = Color::WHITE;
            }
            CurrentState::Attacked { remaining } => {
                let t = *remaining * ATTACKED_EFFECT_CYCLE;
                let fill = 0.5 * t.cos() + 0.5;
                material.base_color = Color::srgb(fill, fill, fill);
            }
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_player_effect_recursive(child, children_query, material_query, materials, state);
        }
    }
}
