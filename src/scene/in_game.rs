// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{asset::spawner::SpawnModel, collider::Collider};

use super::*;

// --- SETUP SYSTEM ---

/// A system that sets up the initial game world.
pub fn on_enter(mut commands: Commands) {
    // --- Resource Initialization ---
    // Insert resources required for the game state.
    commands.insert_resource(PlayScore::default()); // Tracks the player's score.
    commands.insert_resource(InputDelay::default()); // Prevents rapid lane changes.
    commands.insert_resource(RetiredGrounds::default()); // A queue for recycling ground entities.
    commands.insert_resource(ObstacleSpawnTimer::default()); // Timer for spawning new obstacles.

    // --- Player Spawn ---
    // Spawn the main player controller entity. This entity itself is invisible
    // but holds the core movement logic and components.
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, -7.5),
            Lane::default(),             // The player's current lane.
            ForwardMovement::default(),  // Controls forward speed.
            VerticalMovement::default(), // Controls jumping and gravity.
            InGameStateEntity,           // Marker for game-specific entities.
            Player,                      // Marker for the player entity.
        ))
        .with_children(|parent| {
            // Spawn the player's collider as a child.
            parent.spawn((
                Transform::from_xyz(0.0, 0.5, -1.5),
                Collider::Aabb {
                    center: Vec3::new(0.0, 0.0, 0.0),
                    size: Vec3::new(0.9, 1.0, 3.6),
                },
                InGameStateEntity,
                PlayerCollider, // Marker for the player's collider.
            ));
        });

    // --- Lighting ---
    // Spawn a directional light to illuminate the scene.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0, // A bright, sun-like light.
            shadows_enabled: true, // Enable shadows for realism.
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        InGameStateEntity,
    ));

    // --- Camera Spawn ---
    // Spawn the 3D camera with an orthographic projection for a stylized look.
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            near: 0.1,
            far: 100.0,
            // Use a fixed scaling mode to ensure the view is consistent across different window sizes.
            scaling_mode: ScalingMode::Fixed {
                width: 16.0,
                height: 9.0,
            },
            scale: 1.25, // Zoom out slightly.
            ..OrthographicProjection::default_3d()
        }),
        // Position the camera and make it look at a point slightly above the origin.
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
    commands.remove_resource::<PlayScore>();
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

/// A system that updates the player's score based on elapsed time.
pub fn update_score(mut score: ResMut<PlayScore>, time: Res<Time>) {
    score.timer += (time.delta_secs() * 1000.0).floor() as u32;
    if score.timer >= SCORE_CYCLE {
        score.accum = score.accum.saturating_add(score.timer / SCORE_CYCLE);
        score.timer %= SCORE_CYCLE;
    }
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

/// A system that updates the position of all colliders to match their entity's `GlobalTransform`.
/// This ensures that the collider's position is always in sync with the entity's world position.
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
/// This system creates a chain-like movement where each train car follows the one in front of it.
#[allow(clippy::type_complexity)]
pub fn update_toy_trains(
    mut set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<ToyTrain0>>,
        Query<&mut Transform, With<ToyTrain1>>,
        Query<&mut Transform, With<ToyTrain2>>,
    )>,
) {
    // Get the player's controller position. This will be the target for the first train car.
    let data = set
        .p0()
        .single()
        .map(|transform| transform.translation.with_z(transform.translation.z + 1.5))
        .ok();

    if let Some(mut position) = data {
        // --- Update the first train car (ToyTrain0) ---
        if let Ok(mut transform) = set.p1().single_mut() {
            // Calculate the rotation to make the car look at its target (`position`).
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            // ... (rotation calculation) ...
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            // Store the current position of this car. It will become the target for the *next* car.
            let temp = transform.translation;
            // Update the car's position to follow the target.
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -7.5; // Keep a fixed Z-offset from the player.
            transform.rotation = rotation;
            // The target for the next car is now the old position of this car.
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
            if player_collider.intersects(obstacle_collider) {
                // This is a placeholder for game over logic.
                // In a real game, you would transition to a game over state,
                // show a UI, or trigger some other event.
                println!("GAME OVER!");
            }
        }
    }
}

/// A system that updates the score UI by setting the texture atlas index for each digit.
#[allow(clippy::type_complexity)]
pub fn update_score_ui(
    score: Res<PlayScore>,
    // Use a ParamSet to query for each digit's UI node separately and mutably.
    mut set: ParamSet<(
        Query<&mut ImageNode, With<Score0>>, // 1s place
        Query<&mut ImageNode, With<Score1>>, // 10s place
        Query<&mut ImageNode, With<Score2>>, // 100s place
        Query<&mut ImageNode, With<Score3>>, // 1,000s place
        Query<&mut ImageNode, With<Score4>>, // 10,000s place
        Query<&mut ImageNode, With<Score5>>, // 100,000s place
    )>,
) {
    // Update the 1s place digit.
    if let Ok(mut node) = set.p0().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = (score.accum % 10) as usize;
    }

    // Update the 10s place digit.
    if let Ok(mut node) = set.p1().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.accum / 10) % 10) as usize;
    }

    // Update the 100s place digit.
    if let Ok(mut node) = set.p2().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.accum / 100) % 10) as usize;
    }

    // Update the 1,000s place digit.
    if let Ok(mut node) = set.p3().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.accum / 1_000) % 10) as usize;
    }

    // Update the 10,000s place digit.
    if let Ok(mut node) = set.p4().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.accum / 10_000) % 10) as usize;
    }

    // Update the 100,000s place digit.
    if let Ok(mut node) = set.p5().single_mut()
        && let Some(atlas) = &mut node.texture_atlas
    {
        // The index in the texture atlas corresponds to the digit (0-9).
        atlas.index = ((score.accum / 100_000) % 10) as usize;
    }
}
