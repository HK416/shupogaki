// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{asset::spawner::SpawnModel, collider::Collider};

use super::*;

// --- SETUP SYSTEM ---

/// A system that sets up the initial game world.
// NOTE: The ground is no longer spawned here, but is pre-spawned in the `in_game_load` scene.
pub fn on_enter(mut commands: Commands) {
    // Insert resources
    commands.insert_resource(InputDelay::default());
    commands.insert_resource(RetiredGrounds::default());
    commands.insert_resource(ObstacleSpawnTimer::default());

    // Spawn the player entity.
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, -7.5),
            Lane::default(),
            ForwardMovement::default(),
            VerticalMovement::default(),
            InGameStateEntity,
            Player,
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0.0, 0.5, -1.5),
                Collider::Aabb {
                    center: Vec3::new(0.0, 0.0, 0.0),
                    size: Vec3::new(0.9, 1.0, 3.6),
                },
                InGameStateEntity,
                PlayerCollider,
            ));
        });

    // Spawn a directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        InGameStateEntity,
    ));

    // Spawn the 3D camera.
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
        InGameStateEntity,
    ));
}

/// A system that plays the animation for entities with an `AnimationClipHandle`.
/// This system is separate from the entity spawning to ensure that the animation graph is correctly setup after the model has been loaded.
pub fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<(Entity, &AnimationClipHandle)>,
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

// --- CLEANUP SYSTEM ---

/// A system that cleans up the game world when exiting the state.
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<InGameStateEntity>>) {
    // Despawn all entities associated with the InGame state.
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove resources specific to the InGame state.
    commands.remove_resource::<CachedObstacles>();
    commands.remove_resource::<RetiredGrounds>();
    commands.remove_resource::<CachedGrounds>();
    commands.remove_resource::<ObstacleSpawnTimer>();
    commands.remove_resource::<InputDelay>();
}

// --- PREUPDATE SYSTEMS ---

/// A system that handles player input for movement and jumping.
pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Lane, &mut VerticalMovement), With<Player>>,
    mut delay: ResMut<InputDelay>,
) {
    if let Ok((transform, mut lane, mut vert_move)) = query.single_mut() {
        // Check if the input delay has passed and keys are not pressed simultaneously.
        if delay.remaining <= 0.0 && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD]) {
            // Move left.
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.index = lane.index.saturating_sub(1);
                delay.remaining = INPUT_DELAY;
            }
            // Move right.
            else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.index = lane.index.saturating_add(1).min(MAX_LANE_INDEX);
                delay.remaining = INPUT_DELAY;
            }
        }

        // Check if the player is on the ground.
        let is_grounded = transform.translation.y <= 0.0;
        // Jump if the space key is pressed and the player is on the ground.
        if keyboard_input.just_pressed(KeyCode::Space) && is_grounded {
            vert_move.velocity = JUMP_STRENGTH;
        }
    }
}

// --- UPDATE SYSTEMS ---

/// A system that decrements timers for input delay and obstacle spawning.
pub fn update_timer(
    mut delay: ResMut<InputDelay>,
    mut spawn_timer: ResMut<ObstacleSpawnTimer>,
    time: Res<Time>,
) {
    delay.remaining = (delay.remaining - time.delta_secs()).max(0.0);
    spawn_timer.remaining -= time.delta_secs();
}

/// A system that smoothly updates the player's position based on lane and vertical movement.
pub fn update_player_position(
    mut query: Query<(&mut Transform, &Lane, &mut VerticalMovement), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, lane, mut vert_move)) = query.single_mut() {
        // Calculate the target x-position based on the current lane.
        let target_x = LANE_LOCATIONS[lane.index];
        // Smoothly interpolate the player's x-position towards the target.
        transform.translation.x +=
            (target_x - transform.translation.x) * LANE_CHANGE_SPEED * time.delta_secs();

        // Apply gravity to the vertical velocity.
        vert_move.velocity += GRAVITY * time.delta_secs();
        // Update the player's y-position based on the vertical velocity.
        transform.translation.y += vert_move.velocity * time.delta_secs();

        // Prevent the player from falling through the ground.
        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            vert_move.velocity = 0.0;
        }
    }
}

/// A system that moves the ground planes towards the player and recycles them.
pub fn update_ground_position(
    mut commands: Commands,
    player_query: Query<&ForwardMovement, With<Player>>,
    mut ground_query: Query<(Entity, &mut Transform), With<Ground>>,
    mut retired: ResMut<RetiredGrounds>,
    time: Res<Time>,
) {
    if let Ok(forward_move) = player_query.single() {
        for (entity, mut transform) in ground_query.iter_mut() {
            // Move the ground towards the player.
            transform.translation.z -= forward_move.velocity * time.delta_secs();

            // If the ground is off-screen, despawn it and add its transform to the retired queue.
            if transform.translation.z <= -50.0 {
                retired.transforms.push_back(*transform);
                commands.entity(entity).despawn();
            }
        }
    }
}

/// A system that moves obstacles towards the player and despawns them when they are off-screen.
pub fn update_obstacle_position(
    mut commands: Commands,
    player_query: Query<&ForwardMovement, With<Player>>,
    mut obstacle_query: Query<(Entity, &mut Transform), With<Obstacle>>,
    time: Res<Time>,
) {
    if let Ok(forward_move) = player_query.single() {
        for (entity, mut transform) in obstacle_query.iter_mut() {
            // Move the obstacle towards the player.
            transform.translation.z -= forward_move.velocity * time.delta_secs();

            // If the obstacle is off-screen, despawn it.
            if transform.translation.z <= -50.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn update_collider(mut query: Query<(&mut Collider, &GlobalTransform)>) {
    for (mut collider, transform) in query.iter_mut() {
        match collider.as_mut() {
            Collider::Aabb { center, .. } => {
                *center = transform.translation();
            }
            Collider::Sphere { center, .. } => {
                *center = transform.translation();
            }
        }
    }
}

// --- POSTUPDATE SYSTEMS ---

/// A system that updates the positions and rotations of the toy train cars.
///
/// This system creates a chain-like movement where each train car follows the one in front of it.
/// The first car follows the player's invisible controller entity.
pub fn update_toy_trains(
    mut set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<ToyTrain0>>,
        Query<&mut Transform, With<ToyTrain1>>,
        Query<&mut Transform, With<ToyTrain2>>,
    )>,
) {
    // Get the player's target position, which is slightly ahead of the player's actual position.
    let data = set
        .p0()
        .single()
        .map(|transform| transform.translation.with_z(transform.translation.z + 1.5))
        .ok();

    if let Some(mut position) = data {
        // Update the first train car.
        if let Ok(mut transform) = set.p1().single_mut() {
            // Calculate the rotation to make the car look at the target position.
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            // Store the current position to be used as the target for the next car.
            let temp = transform.translation;
            // Move the car to the target position.
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -7.5;
            transform.rotation = rotation;
            // Update the target position for the next car.
            position = temp;
        }

        // Update the second train car.
        if let Ok(mut transform) = set.p2().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            let temp = transform.translation;
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -9.0;
            transform.rotation = rotation;
            position = temp;
        }

        // Update the third train car.
        if let Ok(mut transform) = set.p3().single_mut() {
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -10.5;
            transform.rotation = rotation;
        }
    }
}

/// A system that spawns new ground entities to create an infinite scrolling effect.
/// This system reuses ground entities that have moved off-screen.
/// It now spawns ground models from the `CachedGrounds` resource using a `SpawnModel` command.
pub fn spawn_grounds(
    mut commands: Commands,
    mut retired: ResMut<RetiredGrounds>,
    cached_grounds: Res<CachedGrounds>,
) {
    const MODELS: [GroundModel; 1] = [GroundModel::Plane0];

    while let Some(mut transform) = retired.transforms.pop_front() {
        // Use thread_rng for better random number generation.
        let model = MODELS[rand::random_range(0..MODELS.len())];
        let model_handle = cached_grounds.models.get(&model).unwrap();
        transform.translation.z += 150.0;

        commands.spawn((
            SpawnModel(model_handle.clone()),
            transform,
            InGameStateEntity,
            Ground,
        ));
    }
}

/// A system that spawns obstacles periodically.
pub fn spawn_obstacles(
    mut commands: Commands,
    mut spawn_timer: ResMut<ObstacleSpawnTimer>,
    player_query: Query<&ForwardMovement, With<Player>>,
    cached: Res<CachedObstacles>,
) {
    if let Ok(forward_move) = player_query.single() {
        while spawn_timer.remaining <= 0.0 {
            let time_t = -spawn_timer.remaining;

            // Choose a random lane for the obstacle.
            let lane_index = rand::random_range(0..NUM_LANES);
            let lane_x = LANE_LOCATIONS[lane_index];

            let model_handle = cached.models.get(&ObstacleModel::Rail0).unwrap();

            // Spawn the obstacle entity.
            commands
                .spawn((
                    SpawnModel(model_handle.clone()),
                    Transform::from_xyz(lane_x, 0.0, 100.0 - forward_move.velocity * time_t),
                    InGameStateEntity,
                    Obstacle,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Transform::from_xyz(0.0, 0.5, 0.0),
                        Collider::Aabb {
                            center: Vec3::default(),
                            size: Vec3::splat(1.0),
                        },
                        InGameLoadStateEntity,
                        ObstacleCollider,
                    ));
                });

            spawn_timer.remaining += SPAWN_DELAY;
        }
    }
}

/// A system that checks for collisions between the player and obstacles.
pub fn check_for_collisions(
    player_query: Query<&Collider, (With<PlayerCollider>, Without<ObstacleCollider>)>,
    obstacle_query: Query<&Collider, (With<ObstacleCollider>, Without<PlayerCollider>)>,
) {
    if let Ok(player_collider) = player_query.single() {
        for obstacle_collider in obstacle_query.iter() {
            // A simple collision check based on distance.
            if player_collider.intersects(obstacle_collider) {
                println!("GAME OVER!");
                // In a real game, you would transition to a game over state here.
            }
        }
    }
}
