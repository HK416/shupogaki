// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};
use bevy_tweening::{Animator, TweenCompleted};
use rand::seq::IndexedRandom;

use crate::asset::sound::SystemVolume;

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
        fuel.remaining = 100.0;
    } else if keyboard_input.just_pressed(KeyCode::F7) {
        fuel.remaining = 0.0;
    }
}

pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut lane: ResMut<CurrentLane>,
    mut delay: ResMut<InputDelay>,
    mut vert_move: ResMut<VerticalMovement>,
    mut is_jumping: ResMut<IsPlayerJumping>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(transform) = player_query.single() {
        if delay.is_expired() && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD]) {
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.index = lane.index.saturating_sub(1);
                delay.reset();
            } else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.index = lane.index.saturating_add(1).min(MAX_LANE_INDEX);
                delay.reset();
            }
        }

        let is_grounded = transform.translation.y <= 0.0;
        if is_grounded && keyboard_input.pressed(KeyCode::Space) {
            vert_move.jump();
            is_jumping.jump();
        }
    }
}

pub fn handle_player_input_for_moblie(
    windows: Query<&Window>,
    touches: Res<Touches>,
    mut lane: ResMut<CurrentLane>,
    mut delay: ResMut<InputDelay>,
    mut vert_move: ResMut<VerticalMovement>,
    mut is_jumping: ResMut<IsPlayerJumping>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(window) = windows.single() else { return };
    let window_width = window.width();
    let window_height = window.height();

    let Ok(transform) = player_query.single() else {
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
                    lane.index = lane.index.saturating_sub(1);
                    delay.reset();
                }
            }
            (0.0..=1.0, 0.3..=0.7) => {
                if is_grounded {
                    vert_move.jump();
                    is_jumping.jump();
                }
            }
            (0.3..=0.7, 0.7..=1.0) => {
                if delay.is_expired() {
                    lane.index = lane.index.saturating_add(1).min(MAX_LANE_INDEX);
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
        _ => { /* empty */ }
    }
}

fn update_score(
    mut score: ResMut<CurrentScore>,
    forward_move: Res<ForwardMovement>,
    time: Res<Time>,
) {
    score.on_advanced(&forward_move, time.delta_secs());
}

fn update_train_fuel(
    mut next_state: ResMut<NextState<GameState>>,
    mut fuel: ResMut<TrainFuel>,
    state: Res<CurrentState>,
    time: Res<Time>,
) {
    if !state.is_invincible() {
        fuel.sub(time.delta_secs() * FUEL_USAGE);
    }

    if fuel.is_empty() {
        next_state.set(GameState::WrapUpInGame);
    }
}

fn update_player_position(
    lane: Res<CurrentLane>,
    mut vert_move: ResMut<VerticalMovement>,
    mut is_jumping: ResMut<IsPlayerJumping>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player_query.single_mut() {
        let target_x = LANE_LOCATIONS[lane.index];
        transform.translation.x +=
            (target_x - transform.translation.x) * LANE_CHANGE_SPEED * time.delta_secs();

        vert_move.on_advanced(time.delta_secs());
        transform.translation.y += vert_move.velocity * time.delta_secs();

        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            vert_move.reset();
            is_jumping.reset();
        }
    }
}

fn update_ground_position(
    forward_move: Res<ForwardMovement>,
    mut ground_entities: Query<(Entity, &mut Transform), With<Ground>>,
    mut retired: ResMut<RetiredGrounds>,
    time: Res<Time>,
) {
    for (entity, mut transform) in ground_entities.iter_mut() {
        transform.translation.z -= forward_move.velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_LOCATION {
            retired.push(entity);
        }
    }
}

fn update_object_position(
    mut commands: Commands,
    forward_move: Res<ForwardMovement>,
    mut object_spawner: ResMut<ObjectSpawner>,
    mut object_entities: Query<(Entity, &mut Transform, &Object)>,
    time: Res<Time>,
) {
    for (entity, mut transform, &obj) in object_entities.iter_mut() {
        transform.translation.z -= forward_move.velocity * time.delta_secs();

        if transform.translation.z <= DESPAWN_LOCATION {
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
    forward_move: Res<ForwardMovement>,
    removed: RemovedComponents<TrainSoundStart>,
) {
    if !removed.is_empty() {
        let t = forward_move.percentage();

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
    forward_move: Res<ForwardMovement>,
    removed: RemovedComponents<TrainSoundStart>,
) {
    if !removed.is_empty() {
        let t = forward_move.percentage();

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
    forward_move: Res<ForwardMovement>,
    is_jumping: Res<IsPlayerJumping>,
    mut set: ParamSet<(
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop1>>,
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop2>>,
    )>,
) {
    if !is_jumping.changed() {
        return;
    }

    if is_jumping.get() {
        if let Ok(mut settings) = set.p0().single_mut() {
            *settings = settings.with_volume(Volume::Linear(0.0));
        }

        if let Ok(mut settings) = set.p1().single_mut() {
            *settings = settings.with_volume(Volume::Linear(0.0));
        }
    } else {
        if let Ok(mut settings) = set.p0().single_mut() {
            let t = forward_move.percentage();
            let volume = system_volume.effect_percentage() * (1.0 - t);
            *settings = settings.with_volume(Volume::Linear(volume));
        }

        if let Ok(mut settings) = set.p1().single_mut() {
            let t = forward_move.percentage();
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
    forward_move: Res<ForwardMovement>,
    mut set: ParamSet<(
        Query<&mut AudioSink, With<TrainSoundLoop1>>,
        Query<&mut AudioSink, With<TrainSoundLoop2>>,
    )>,
) {
    if let Ok(mut sink) = set.p0().single_mut() {
        let t = forward_move.percentage();
        let volume = system_volume.effect_percentage() * (1.0 - t);
        sink.set_volume(Volume::Linear(volume));
    }

    if let Ok(mut sink) = set.p1().single_mut() {
        let t = forward_move.percentage();
        let volume = system_volume.effect_percentage() * t;
        sink.set_volume(Volume::Linear(volume));
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::type_complexity)]
fn update_train_volume(
    system_volume: Res<SystemVolume>,
    forward_move: Res<ForwardMovement>,
    is_jumping: Res<IsPlayerJumping>,
    mut set: ParamSet<(
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop1>>,
        Query<&mut WebPlaybackSettings, With<TrainSoundLoop2>>,
    )>,
) {
    if is_jumping.get() {
        return;
    }

    if let Ok(mut settings) = set.p0().single_mut() {
        let t = forward_move.percentage();
        let volume = system_volume.effect_percentage() * (1.0 - t);
        *settings = settings.with_volume(Volume::Linear(volume));
    }

    if let Ok(mut settings) = set.p1().single_mut() {
        let t = forward_move.percentage();
        let volume = system_volume.effect_percentage() * t;
        *settings = settings.with_volume(Volume::Linear(volume));
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
                transform.translation.z += SPAWN_LOCATION - DESPAWN_LOCATION;
            })
            .or_insert(Transform::from_xyz(0.0, 0.0, SPAWN_LOCATION));
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

#[allow(clippy::too_many_arguments)]
fn check_for_collisions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    mut fuel: ResMut<TrainFuel>,
    mut state: ResMut<CurrentState>,
    mut attacked: ResMut<Attacked>,
    mut forward_move: ResMut<ForwardMovement>,
    mut player_query: Query<(&Collider, &Transform), With<Player>>,
    object_query: Query<(Entity, &Object, &Collider, &Transform)>,
) {
    for (entity, object, o_collider, o_trans) in object_query.iter() {
        if let Ok((p_collider, p_trans)) = player_query.single_mut()
            && p_collider.intersects(p_trans, o_collider, o_trans)
        {
            info!("Collision detected!");
            match (*state, *object) {
                (CurrentState::Idle, Object::Barricade) => {
                    play_damaged_sound(&mut commands, &asset_server, &system_volume);
                    fuel.sub(BARRICADE_AMOUNT);
                    forward_move.reset();
                    attacked.add();
                    *state = CurrentState::Attacked {
                        remaining: ATTACKED_DURATION,
                    };
                }
                (CurrentState::Idle, Object::Stone) => {
                    play_damaged_sound(&mut commands, &asset_server, &system_volume);
                    fuel.sub(STONE_AMOUNT);
                    forward_move.reset();
                    attacked.add();
                    *state = CurrentState::Attacked {
                        remaining: ATTACKED_DURATION,
                    };
                }
                (CurrentState::Idle, Object::Fuel) => {
                    play_healing_sound(&mut commands, &asset_server, &system_volume);
                    fuel.add(FUEL_AMOUNT);
                    commands.entity(entity).despawn();
                }
                (CurrentState::Attacked { .. }, Object::Fuel) => {
                    fuel.add(FUEL_AMOUNT);
                    commands.entity(entity).despawn();
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
        node.width = Val::Percent(fuel.remaining);
        color.0 = match fuel.remaining {
            50.0..=100.0 => FUEL_GOOD_GAUGE_COLOR,
            25.0..=50.0 => FUEL_FAIR_GAUGE_COLOR,
            _ => FUEL_POOR_GAUGE_COLOR,
        };
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

pub fn update_player_speed(mut forward_move: ResMut<ForwardMovement>, time: Res<Time>) {
    forward_move.accel(time.delta_secs());
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
