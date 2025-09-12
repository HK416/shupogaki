// Import necessary Bevy modules.
use bevy::prelude::*;

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
        next_state.set(GameState::CleanUpInGame);
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

pub fn update_player_speed(mut forward_move: ResMut<ForwardMovement>, time: Res<Time>) {
    forward_move.decel(time.delta_secs());
}
