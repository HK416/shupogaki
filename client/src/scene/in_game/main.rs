use std::f32::consts::TAU;

// Import necessary Bevy modules.
use bevy::{
    audio::Volume,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_tweening::{Animator, TweenCompleted};
use rand::seq::IndexedRandom;

use crate::{
    asset::{animation::AnimationClipHandle, material::EyeMouthMaterial, sound::SystemVolume},
    collider::Collider,
};

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), debug_label)
            .add_systems(OnExit(GameState::InGame), hide_in_game_interface)
            .add_systems(
                PreUpdate,
                (
                    #[cfg(not(feature = "no-debuging-player"))]
                    {
                        handle_player
                    },
                    handle_player_input,
                    handle_player_input_for_moblie,
                    handle_pause_input,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    update_play_time,
                    update_input_delay,
                    update_player_state,
                    update_score,
                    update_train_fuel,
                    update_player_position,
                    update_ground_position,
                    update_object_position,
                    play_aoba_animation.after(update_object_position),
                    setup_no_shadow_casting,
                    rotate_animation,
                    fade_in_out_animation,
                    cleanup_ui_animation,
                    button_system,
                    play_train_sound,
                    update_train_sound,
                    update_train_volume,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                (
                    update_toy_trains,
                    spawn_grounds,
                    spawn_objects,
                    check_for_collisions,
                    update_score_ui,
                    update_fuel_deco,
                    update_fuel_gauge,
                    update_player_effect,
                    update_player_speed,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: InGame");
}

// --- CLEANUP SYSTEMS ---

fn hide_in_game_interface(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, &ui) in query.iter_mut() {
        match ui {
            UI::StartLabel => *visibility = Visibility::Hidden,
            _ => { /* empty */ }
        }
    }
}

// --- PREUPDATE SYSTEMS ---
#[cfg(not(feature = "no-debuging-player"))]
pub fn handle_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CurrentState>,
    mut fuel: ResMut<TrainFuel>,
) {
    if keyboard_input.just_pressed(KeyCode::F5) {
        *state = if state.is_debug() {
            CurrentState::Idle
        } else {
            CurrentState::Debug
        };
    } else if keyboard_input.just_pressed(KeyCode::F6) {
        fuel.set(100.0);
    } else if keyboard_input.just_pressed(KeyCode::F7) {
        fuel.set(0.0);
    }
}

pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut delay: ResMut<InputDelay>,
    mut is_jumping: ResMut<IsPlayerJumping>,
    mut player_query: Query<(&mut Lane, &Transform, &mut VerticalMovement), With<Player>>,
) {
    if let Ok((mut lane, transform, mut vert_move)) = player_query.single_mut() {
        if delay.is_expired() && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD]) {
            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                lane.dec();
                delay.reset();
            } else if keyboard_input.pressed(KeyCode::KeyD)
                || keyboard_input.pressed(KeyCode::ArrowRight)
            {
                lane.inc();
                delay.reset();
            }
        }

        let is_grounded = transform.translation.y <= 0.0;
        if is_grounded && keyboard_input.pressed(KeyCode::Space) {
            vert_move.set(JUMP_STRENGTH);
            is_jumping.jump();
        }
    }
}

pub fn handle_player_input_for_moblie(
    windows: Query<&Window>,
    touches: Res<Touches>,
    mut delay: ResMut<InputDelay>,
    mut is_jumping: ResMut<IsPlayerJumping>,
    mut player_query: Query<(&mut Lane, &Transform, &mut VerticalMovement), With<Player>>,
) {
    let Ok(window) = windows.single() else { return };
    let window_width = window.width();
    let window_height = window.height();

    let Ok((mut lane, transform, mut vert_move)) = player_query.single_mut() else {
        return;
    };
    let is_grounded = transform.translation.y <= 0.0;

    for touch in touches.iter_just_pressed() {
        let position = touch.position();
        let p_vertical = position.y / window_height;
        let p_horizontal = position.x / window_width;
        match (p_vertical, p_horizontal) {
            (0.3..=0.7, 0.0..=0.3) => {
                if delay.is_expired() {
                    lane.dec();
                    delay.reset();
                }
            }
            (0.0..=1.0, 0.3..=0.7) => {
                if is_grounded {
                    vert_move.set(JUMP_STRENGTH);
                    is_jumping.jump();
                }
            }
            (0.3..=0.7, 0.7..=1.0) => {
                if delay.is_expired() {
                    lane.inc();
                    delay.reset();
                }
            }
            _ => { /* empty */ }
        }
    }
}

fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Pause);
    }
}

// --- UPDATE SYSTEMS ---

fn update_play_time(mut timer: ResMut<PlayTime>, time: Res<Time>) {
    timer.tick(&time);
}

fn update_input_delay(mut delay: ResMut<InputDelay>, time: Res<Time>) {
    delay.on_advanced(time.delta_secs());
}

fn update_player_state(mut state: ResMut<CurrentState>, time: Res<Time>) {
    match &mut *state {
        CurrentState::Attacked { remaining } => {
            *remaining -= time.delta_secs();
            if *remaining <= 0.0 {
                *state = CurrentState::Idle;
            }
        }
        CurrentState::Invincible { remaining } => {
            *remaining -= time.delta_secs();
            if *remaining <= 0.0 {
                *state = CurrentState::Attacked {
                    remaining: ATTACKED_DURATION,
                };
            }
        }
        _ => { /* empty */ }
    }
}

fn update_score(
    mut score: ResMut<CurrentScore>,
    player_query: Query<&ForwardMovement, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(forward_move) = player_query.single() {
        score.on_advanced(forward_move, time.delta_secs());
    }
}

fn update_train_fuel(
    mut next_state: ResMut<NextState<GameState>>,
    mut fuel: ResMut<TrainFuel>,
    state: Res<CurrentState>,
    time: Res<Time>,
) {
    if !state.is_invincible() {
        fuel.dec(time.delta_secs() * FUEL_USAGE);
    }

    if fuel.is_empty() {
        next_state.set(GameState::WrapUpInGame);
    }
}

fn update_player_position(
    mut is_jumping: ResMut<IsPlayerJumping>,
    mut player_query: Query<(&Lane, &mut Transform, &mut VerticalMovement), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((lane, mut transform, mut vert_move)) = player_query.single_mut() {
        let target_x = LANE_POSITIONS[lane.get()];
        transform.translation.x +=
            (target_x - transform.translation.x) * LANE_SWITCH_SPEED * time.delta_secs();

        let mut velocity = vert_move.get();
        velocity += GRAVITY * time.delta_secs();
        vert_move.set(velocity);

        transform.translation.y += vert_move.get() * time.delta_secs();
        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            vert_move.set(0.0);
            is_jumping.reset();
        }
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
    mut object_spawner: ResMut<ObjectSpawner>,
    mut object_entities: Query<(Entity, &mut Transform, &Object)>,
    player_query: Query<&ForwardMovement, With<Player>>,
    time: Res<Time>,
) {
    let player_velocity = player_query
        .single()
        .map(|forward_move| forward_move.get())
        .unwrap_or(0.0);

    for (entity, mut transform, &obj) in object_entities.iter_mut() {
        transform.translation.z -= player_velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_POSITION {
            object_spawner.drain(&mut commands, entity, obj);
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

fn fade_in_out_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImageNode, &mut FadeInOutAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut image_node, mut fade_in_out) in query.iter_mut() {
        fade_in_out.tick(time.delta_secs());
        if fade_in_out.is_expired() {
            commands.entity(entity).despawn();
        } else {
            image_node.color = fade_in_out.color();
        }
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

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: ResMut<SystemVolume>,
    mut query: Query<
        (&UI, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (ui, interaction, mut color) in query.iter_mut() {
        match (*ui, *interaction) {
            (UI::PauseButton, Interaction::Hovered) => {
                color.0 = PAUSE_BTN_COLOR.darker(0.25);
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::PauseButton, Interaction::Pressed) => {
                color.0 = PAUSE_BTN_COLOR.darker(0.5);
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                next_state.set(GameState::Pause);
            }
            (UI::PauseButton, Interaction::None) => {
                color.0 = PAUSE_BTN_COLOR;
            }
            _ => { /* empty */ }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_train_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    player_query: Query<&ForwardMovement, With<Player>>,
    removed: RemovedComponents<TrainSoundStart>,
) {
    if !removed.is_empty()
        && let Ok(forward_move) = player_query.single()
    {
        let velocity = forward_move.get();
        let t = (velocity - MIN_PLAYER_SPEED) / (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED);
        let t = t.clamp(0.0, 1.0);

        let volume = system_volume.effect_percentage() * (1.0 - t);
        commands.spawn((
            AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_1)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            TrainSoundLoop1,
            InGameStateRoot,
            EffectSound,
        ));

        let volume = system_volume.effect_percentage() * t;
        commands.spawn((
            AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_2)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            TrainSoundLoop2,
            InGameStateRoot,
            EffectSound,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
fn play_train_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    player_query: Query<&ForwardMovement, With<Player>>,
    removed: RemovedComponents<TrainSoundStart>,
) {
    if !removed.is_empty()
        && let Ok(forward_move) = player_query.single()
    {
        let velocity = forward_move.get();
        let t = (velocity - MIN_PLAYER_SPEED) / (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED);
        let t = t.clamp(0.0, 1.0);

        let volume = system_volume.effect_percentage() * (1.0 - t);
        commands.spawn((
            WebAudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_1)),
            WebPlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            TrainSoundLoop1,
            InGameStateRoot,
            EffectSound,
        ));

        let volume = system_volume.effect_percentage() * t;
        commands.spawn((
            WebAudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LOOP_2)),
            WebPlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            TrainSoundLoop2,
            InGameStateRoot,
            EffectSound,
        ));
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::type_complexity)]
fn update_train_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    is_jumping: Res<IsPlayerJumping>,
    mut set: ParamSet<(
        Query<&mut AudioSink, With<TrainSoundLoop1>>,
        Query<&mut AudioSink, With<TrainSoundLoop2>>,
    )>,
) {
    if !is_jumping.changed() {
        return;
    }

    if is_jumping.get() {
        if let Ok(mut sink) = set.p0().single_mut() {
            sink.mute();
        }

        if let Ok(mut sink) = set.p1().single_mut() {
            sink.mute();
        }
    } else {
        if let Ok(mut sink) = set.p0().single_mut() {
            sink.unmute();
        }

        if let Ok(mut sink) = set.p1().single_mut() {
            sink.unmute();
        }

        commands.spawn((
            AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LANDING)),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(system_volume.effect_percentage())),
            InGameStateRoot,
            EffectSound,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::type_complexity)]
fn update_train_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    player_query: Query<&ForwardMovement, With<Player>>,
    is_jumping: Res<IsPlayerJumping>,
    mut set: ParamSet<(
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop1>>,
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop2>>,
    )>,
) {
    if !is_jumping.changed() {
        return;
    }

    let Ok(forward_move) = player_query.single() else {
        return;
    };
    let velocity = forward_move.get();
    let t = (velocity - MIN_PLAYER_SPEED) / (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED);
    let t = t.clamp(0.0, 1.0);

    if is_jumping.get() {
        if let Ok(mut settings) = set.p0().single_mut() {
            *settings = settings.with_volume(Volume::Linear(0.0));
        }

        if let Ok(mut settings) = set.p1().single_mut() {
            *settings = settings.with_volume(Volume::Linear(0.0));
        }
    } else {
        if let Ok(mut settings) = set.p0().single_mut() {
            let volume = system_volume.effect_percentage() * (1.0 - t);
            *settings = settings.with_volume(Volume::Linear(volume));
        }

        if let Ok(mut settings) = set.p1().single_mut() {
            let volume = system_volume.effect_percentage() * t;
            *settings = settings.with_volume(Volume::Linear(volume));
        }

        commands.spawn((
            WebAudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_LANDING)),
            WebPlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(system_volume.effect_percentage())),
            InGameStateRoot,
            EffectSound,
        ));
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::type_complexity)]
fn update_train_volume(
    system_volume: Res<SystemVolume>,
    player_query: Query<&ForwardMovement, With<Player>>,
    mut set: ParamSet<(
        Query<&mut AudioSink, With<TrainSoundLoop1>>,
        Query<&mut AudioSink, With<TrainSoundLoop2>>,
    )>,
) {
    if let Ok(forward_move) = player_query.single() {
        let velocity = forward_move.get();
        let t = (velocity - MIN_PLAYER_SPEED) / (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED);
        let t = t.clamp(0.0, 1.0);

        if let Ok(mut sink) = set.p0().single_mut() {
            let volume = system_volume.effect_percentage() * (1.0 - t);
            sink.set_volume(Volume::Linear(volume));
        }

        if let Ok(mut sink) = set.p1().single_mut() {
            let volume = system_volume.effect_percentage() * t;
            sink.set_volume(Volume::Linear(volume));
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::type_complexity)]
fn update_train_volume(
    system_volume: Res<SystemVolume>,
    player_query: Query<&ForwardMovement, With<Player>>,
    is_jumping: Res<IsPlayerJumping>,
    mut set: ParamSet<(
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop1>>,
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop2>>,
    )>,
) {
    if is_jumping.get() {
        return;
    }

    if let Ok(forward_move) = player_query.single() {
        let velocity = forward_move.get();
        let t = (velocity - MIN_PLAYER_SPEED) / (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED);
        let t = t.clamp(0.0, 1.0);

        if let Ok(mut settings) = set.p0().single_mut() {
            let volume = system_volume.effect_percentage() * (1.0 - t);
            *settings = settings.with_volume(Volume::Linear(volume));
        }

        if let Ok(mut settings) = set.p1().single_mut() {
            let volume = system_volume.effect_percentage() * t;
            *settings = settings.with_volume(Volume::Linear(volume));
        }
    }
}

#[allow(clippy::type_complexity)]
fn play_aoba_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    aoba_query: Query<(Entity, &AnimationClipHandle, &GlobalTransform), With<InGameStateEntity>>,
) {
    for (entity, clip, transform) in aoba_query.iter() {
        if transform.translation().z <= 17.5 {
            let (graph, animation_index) = AnimationGraph::from_clip(clip.0.clone());
            let mut player = AnimationPlayer::default();
            player.play(animation_index).repeat();

            info!("Play Animation!");
            commands
                .entity(entity)
                .insert((AnimationGraphHandle(graphs.add(graph)), player))
                .remove::<AnimationClipHandle>();

            play_aoba_sound(&mut commands, &asset_server, &system_volume);
        }
    }
}

#[allow(clippy::type_complexity)]
fn setup_no_shadow_casting(
    mut commands: Commands,
    children_query: Query<&Children>,
    query: Query<(Entity, &Children), (With<GlowRoot>, Added<Children>)>,
) {
    for (entity, children) in query.iter() {
        for child in children.iter() {
            apply_to_descendants(&mut commands, child, &children_query);
        }

        commands.entity(entity).remove::<GlowRoot>();
    }
}

fn apply_to_descendants(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    commands
        .entity(entity)
        .insert((NotShadowCaster, NotShadowReceiver));

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            apply_to_descendants(commands, child, children_query);
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

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

fn spawn_objects(
    mut commands: Commands,
    mut spawner: ResMut<ObjectSpawner>,
    player_query: Query<&ForwardMovement, With<Player>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    let Ok(forward_move) = player_query.single() else {
        return;
    };

    spawner.on_advanced(
        &mut commands,
        &asset_server,
        forward_move,
        time.delta_secs(),
    );
}

#[allow(clippy::too_many_arguments)]
fn check_for_collisions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mut fuel: ResMut<TrainFuel>,
    mut state: ResMut<CurrentState>,
    mut score: ResMut<CurrentScore>,
    mut attacked: ResMut<Attacked>,
    mut player_query: Query<(&Collider, &Transform, &mut ForwardMovement), With<Player>>,
    object_query: Query<(Entity, &Object, &Collider, &Transform)>,
) {
    for (entity, object, o_collider, o_trans) in object_query.iter() {
        if let Ok((p_collider, p_trans, mut forward_move)) = player_query.single_mut()
            && p_collider.intersects(p_trans, o_collider, o_trans)
        {
            info!("Collision detected!");
            match (*state, *object) {
                (CurrentState::Idle, Object::Barricade) => {
                    play_damaged_sound(&mut commands, &asset_server, &system_volume);
                    fuel.dec(BARRICADE_DAMAGE);
                    forward_move.set(MIN_PLAYER_SPEED);
                    attacked.add();
                    *state = CurrentState::Attacked {
                        remaining: ATTACKED_DURATION,
                    };
                }
                (CurrentState::Idle, Object::Stone) => {
                    play_damaged_sound(&mut commands, &asset_server, &system_volume);
                    fuel.dec(STONE_DAMAGE);
                    forward_move.set(MIN_PLAYER_SPEED);
                    attacked.add();
                    *state = CurrentState::Attacked {
                        remaining: ATTACKED_DURATION,
                    };
                }
                (CurrentState::Idle, Object::Fuel) => {
                    play_healing_sound(&mut commands, &asset_server, &system_volume);
                    fuel.inc(FUEL_HEALING);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Idle, Object::Bell) => {
                    play_door_bell_sound(&mut commands, &asset_server, &system_volume);
                    score.inc(BELL_POINT);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Idle, Object::Aoba) => {
                    play_invincible_sound(&mut commands, &asset_server, &system_volume);
                    forward_move.set(INVINCIBLE_SPEED);
                    commands.entity(entity).despawn();
                    *state = CurrentState::Invincible {
                        remaining: INVINCIBLE_DURATION,
                    };
                }
                (CurrentState::Attacked { .. }, Object::Fuel) => {
                    fuel.inc(FUEL_HEALING);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Attacked { .. }, Object::Bell) => {
                    play_door_bell_sound(&mut commands, &asset_server, &system_volume);
                    score.inc(BELL_POINT);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Attacked { .. }, Object::Aoba) => {
                    play_invincible_sound(&mut commands, &asset_server, &system_volume);
                    forward_move.set(INVINCIBLE_SPEED);
                    commands.entity(entity).despawn();
                    *state = CurrentState::Invincible {
                        remaining: INVINCIBLE_DURATION,
                    };
                }
                (CurrentState::Invincible { .. }, Object::Fuel) => {
                    fuel.inc(FUEL_HEALING);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Invincible { .. }, Object::Bell) => {
                    play_door_bell_sound(&mut commands, &asset_server, &system_volume);
                    score.inc(BELL_POINT);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Invincible { .. }, Object::Aoba) => {
                    play_invincible_sound(&mut commands, &asset_server, &system_volume);
                    forward_move.set(INVINCIBLE_SPEED);
                    commands.entity(entity).despawn();
                    *state = CurrentState::Invincible {
                        remaining: INVINCIBLE_DURATION,
                    };
                }
                _ => { /* empty */ }
            }
            break;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_damaged_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    if rand::random_ratio(2, 3) {
        let path = SOUND_PATH_VO_DAMAGEDS
            .choose(&mut rand::rng())
            .copied()
            .unwrap();
        commands.spawn((
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
            InGameStateRoot,
            VoiceSound,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
fn play_damaged_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    if rand::random_ratio(2, 3) {
        let path = SOUND_PATH_VO_DAMAGEDS
            .choose(&mut rand::rng())
            .copied()
            .unwrap();
        commands.spawn((
            WebAudioPlayer::new(asset_server.load(path)),
            WebPlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(system_volume.voice_percentage())),
            InGameStateRoot,
            VoiceSound,
        ));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_healing_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    if rand::random_ratio(1, 3) {
        let path = SOUND_PATH_VO_HEALINGS
            .choose(&mut rand::rng())
            .copied()
            .unwrap();
        commands.spawn((
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
            InGameStateRoot,
            VoiceSound,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
fn play_healing_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    if rand::random_ratio(1, 3) {
        let path = SOUND_PATH_VO_HEALINGS
            .choose(&mut rand::rng())
            .copied()
            .unwrap();
        commands.spawn((
            WebAudioPlayer::new(asset_server.load(path)),
            WebPlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(system_volume.voice_percentage())),
            InGameStateRoot,
            VoiceSound,
        ));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_door_bell_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_DOOR_BELL)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_door_bell_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_SFX_DOOR_BELL)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn play_aoba_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    let path = SOUND_PATH_VO_AOBAS
        .choose(&mut rand::rng())
        .copied()
        .unwrap();
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_aoba_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    let path = SOUND_PATH_VO_AOBAS
        .choose(&mut rand::rng())
        .copied()
        .unwrap();
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(path)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn play_invincible_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    let mut rng = rand::rng();
    let path = SOUND_PATH_VO_AOBA_HITS.choose(&mut rng).copied().unwrap();
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));

    let path = SOUND_PATH_VO_INVINCIBLES.choose(&mut rng).copied().unwrap();
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));

    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_INVINCIBLE)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_invincible_sound(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    let mut rng = rand::rng();
    let path = SOUND_PATH_VO_AOBA_HITS.choose(&mut rng).copied().unwrap();
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(path)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));

    let path = SOUND_PATH_VO_INVINCIBLES.choose(&mut rng).copied().unwrap();
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(path)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.voice_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));

    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_SFX_TRAIN_INVINCIBLE)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        InGameStateRoot,
        VoiceSound,
    ));
}

#[allow(clippy::type_complexity)]
fn update_score_ui(
    score: Res<CurrentScore>,
    // Use a ParamSet to query for each digit's UI node separately,
    // as we need mutable access to multiple components that would otherwise conflict.
    mut set: ParamSet<(
        Query<&mut ImageNode, With<ScoreSpace1s>>,      // 1s place
        Query<&mut ImageNode, With<ScoreSpace10s>>,     // 10s place
        Query<&mut ImageNode, With<ScoreSpace100s>>,    // 100s place
        Query<&mut ImageNode, With<ScoreSpace1000s>>,   // 1,000s place
        Query<&mut ImageNode, With<ScoreSpace10000s>>,  // 10,000s place
        Query<&mut ImageNode, With<ScoreSpace100000s>>, // 100,000s place
    )>,
) {
    // Update the 1s place digit.
    if let Ok(mut node) = set.p0().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = (score.get() % 10) as usize;
    }

    // Update the 10s place digit.
    if let Ok(mut node) = set.p1().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.get() / 10) % 10) as usize;
    }

    // Update the 100s place digit.
    if let Ok(mut node) = set.p2().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.get() / 100) % 10) as usize;
    }

    // Update the 1,000s place digit.
    if let Ok(mut node) = set.p3().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.get() / 1_000) % 10) as usize;
    }

    // Update the 10,000s place digit.
    if let Ok(mut node) = set.p4().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.get() / 10_000) % 10) as usize;
    }

    // Update the 100,000s place digit.
    if let Ok(mut node) = set.p5().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.get() / 100_000) % 10) as usize;
    }
}

fn update_fuel_deco(mut query: Query<&mut Node, With<FuelDeco>>, time: Res<Time>) {
    if let Ok(mut node) = query.single_mut() {
        let t = time.elapsed_secs() * FUEL_DECO_CYCLE;
        node.top = Val::Percent(12.5 + 2.5 * t.sin());
    }
}
fn update_fuel_gauge(
    mut query: Query<(&mut Node, &mut BackgroundColor), With<FuelGauge>>,
    fuel: Res<TrainFuel>,
) {
    if let Ok((mut node, mut color)) = query.single_mut() {
        node.width = Val::Percent(fuel.get());
        color.0 = match fuel.get() {
            50.0..=100.0 => FUEL_GOOD_GAUGE_COLOR,
            25.0..=50.0 => FUEL_FAIR_GAUGE_COLOR,
            _ => FUEL_POOR_GAUGE_COLOR,
        };
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn update_player_effect(
    mut set: ParamSet<(
        Query<Entity, With<ToyTrain0>>,
        Query<Entity, With<ToyTrain1>>,
        Query<Entity, With<ToyTrain2>>,
    )>,
    children_query: Query<&Children>,
    base_color_query: Query<&BaseColor>,
    standard_material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    extented_material_query: Query<&MeshMaterial3d<EyeMouthMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut extended_materials: ResMut<Assets<EyeMouthMaterial>>,
    mut state: ResMut<CurrentState>,
) {
    if let Ok(entity) = set.p0().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
            &mut state,
        );
    }

    if let Ok(entity) = set.p1().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
            &mut state,
        );
    }

    if let Ok(entity) = set.p2().single() {
        update_player_effect_recursive(
            entity,
            &children_query,
            &base_color_query,
            &standard_material_query,
            &extented_material_query,
            &mut standard_materials,
            &mut extended_materials,
            &mut state,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn update_player_effect_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    base_color_query: &Query<&BaseColor>,
    standard_material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    extented_material_query: &Query<&MeshMaterial3d<EyeMouthMaterial>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    extended_materials: &mut ResMut<Assets<EyeMouthMaterial>>,
    state: &mut CurrentState,
) {
    if let Ok(handle) = standard_material_query.get(entity)
        && let Some(material) = standard_materials.get_mut(handle.id())
    {
        match &mut *state {
            #[cfg(not(feature = "no-debuging-player"))]
            CurrentState::Debug => {
                material.base_color = Color::BLACK;
            }
            CurrentState::Idle => {
                material.base_color = base_color_query
                    .get(entity)
                    .map(|c| c.0)
                    .unwrap_or(Color::WHITE);
            }
            CurrentState::Attacked { remaining } => {
                let t = *remaining * ATTACKED_EFFECT_CYCLE;
                let fill = 0.5 * t.cos() + 0.5;
                material.base_color = Color::srgba(fill, fill, fill, material.base_color.alpha());
            }
            CurrentState::Invincible { remaining } => {
                let t = ((INVINCIBLE_DURATION - *remaining) / INVINCIBLE_DURATION).max(0.0);
                let cycle =
                    MIN_INVINCIBLE_EFFECT_CYCLE * (1.0 - t) + MAX_INVINCIBLE_EFFECT_CYCLE * t;
                let red = 0.5 * (t * cycle).sin() + 0.5;
                let green = 0.5 * (TAU / 3.0 * t * cycle).sin() + 0.5;
                let blue = 0.5 * (2.0 * TAU / 3.0 * t * cycle).sin() + 0.5;
                material.base_color = Color::srgba(red, green, blue, material.base_color.alpha());
            }
        }
    }

    if let Ok(handle) = extented_material_query.get(entity)
        && let Some(material) = extended_materials.get_mut(handle.id())
    {
        match &mut *state {
            #[cfg(not(feature = "no-debuging-player"))]
            CurrentState::Debug => {
                material.base.base_color = Color::BLACK;
            }
            CurrentState::Idle => {
                material.base.base_color = base_color_query
                    .get(entity)
                    .map(|c| c.0)
                    .unwrap_or(Color::WHITE);
            }
            CurrentState::Attacked { remaining } => {
                let t = *remaining * ATTACKED_EFFECT_CYCLE;
                let fill = 0.5 * t.cos() + 0.5;
                material.base.base_color =
                    Color::srgba(fill, fill, fill, material.base.base_color.alpha());
            }
            CurrentState::Invincible { remaining } => {
                let t = ((INVINCIBLE_DURATION - *remaining) / INVINCIBLE_DURATION).max(0.0);
                let cycle =
                    MIN_INVINCIBLE_EFFECT_CYCLE * (1.0 - t) + MAX_INVINCIBLE_EFFECT_CYCLE * t;
                let red = 0.5 * (t * cycle).sin() + 0.5;
                let green = 0.5 * (TAU / 3.0 * t * cycle).sin() + 0.5;
                let blue = 0.5 * (2.0 * TAU / 3.0 * t * cycle).sin() + 0.5;
                material.base.base_color =
                    Color::srgba(red, green, blue, material.base.base_color.alpha());
            }
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_player_effect_recursive(
                child,
                children_query,
                base_color_query,
                standard_material_query,
                extented_material_query,
                standard_materials,
                extended_materials,
                state,
            );
        }
    }
}

pub fn update_player_speed(
    mut player_query: Query<(&mut ForwardMovement, &Acceleration), With<Player>>,
    state: Res<CurrentState>,
    time: Res<Time>,
) {
    if !matches!(*state, CurrentState::Invincible { .. })
        && let Ok((mut forward_move, accel)) = player_query.single_mut()
    {
        let mut velocity = forward_move.get();
        velocity += accel.get() * time.delta_secs();
        velocity = velocity.min(MAX_PLAYER_SPEED);
        forward_move.set(velocity);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_button_sound_when_hovered(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_button_sound_when_hovered(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn play_button_sound_when_pressed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_TOUCH)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_button_sound_when_pressed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_TOUCH)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}
