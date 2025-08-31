// Import necessary Bevy modules.
use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{asset::spawner::SpawnModel, collider::Collider};

use super::*;

// --- SETUP SYSTEM ---

/// A system that sets up the initial game world.
pub fn on_enter(mut commands: Commands) {
    // --- Resource Initialization ---
    // Insert resources required for the game state.
    commands.insert_resource(TrainFuel::default()); // Manages the player's fuel.
    commands.insert_resource(PlayScore::default()); // Tracks the player's score.
    commands.insert_resource(InputDelay::default()); // Prevents rapid lane changes.
    commands.insert_resource(InvincibleTimer::default()); // Timer for temporary invincibility after a collision.
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
    commands.remove_resource::<InvincibleTimer>();
    commands.remove_resource::<InputDelay>();
    commands.remove_resource::<PlayScore>();
    commands.remove_resource::<TrainFuel>();
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
        if delay.is_expired() && !keyboard_input.all_pressed([KeyCode::KeyA, KeyCode::KeyD]) {
            // Move left.
            if keyboard_input.pressed(KeyCode::KeyA) {
                lane.index = lane.index.saturating_sub(1);
                delay.reset();
            }
            // Move right.
            else if keyboard_input.pressed(KeyCode::KeyD) {
                lane.index = lane.index.saturating_add(1).min(MAX_LANE_INDEX);
                delay.reset();
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
    mut invincible_timer: ResMut<InvincibleTimer>,
    time: Res<Time>,
) {
    delay.on_advanced(time.delta_secs());
    spawn_timer.on_advanced(time.delta_secs());
    invincible_timer.on_advanced(time.delta_secs());
}

/// A system that updates the player's score based on elapsed time.
pub fn update_score(mut score: ResMut<PlayScore>, time: Res<Time>) {
    score.timer += (time.delta_secs() * 1000.0).floor() as u32;
    if score.timer >= SCORE_CYCLE {
        score.accum = score.accum.saturating_add(score.timer / SCORE_CYCLE);
        score.timer %= SCORE_CYCLE;
    }
}

/// A system that consumes fuel over time and triggers a game over when it runs out.
pub fn update_fuel(mut fuel: ResMut<TrainFuel>, time: Res<Time>) {
    // Decrease the fuel based on the time elapsed and the defined usage rate.
    fuel.saturating_sub(time.delta_secs() * FUEL_USAGE);

    // If fuel is empty, the game should end.
    if fuel.is_empty() {
        // TODO: Implement the actual game over logic (e.g., transitioning to a game over screen).
        todo!("Game Over!");
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
        // Smoothly interpolate the player's x-position towards the target lane's x-coordinate.
        // This creates a fluid lane-changing motion instead of an instant snap.
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
            // Calculate the rotation required for the car to "look at" its target (`position`).
            // This creates the snake-like turning effect.
            let z_axis = (transform.translation - position).normalize_or(Vec3::NEG_Z);
            let y_axis = Vec3::Y;
            let x_axis = y_axis.cross(z_axis);
            let y_axis = z_axis.cross(x_axis);
            let rotation = Quat::from_mat3(&Mat3::from_cols(x_axis, y_axis, z_axis));

            // Store the current position of this car. It will become the target for the *next* car in the chain.
            let temp = transform.translation;

            // Update this car's position to follow its target.
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = -7.5; // Keep a fixed Z-offset from the player.
            transform.rotation = rotation;

            // The target for the next car is now the old position of this car. This is the key to the chain movement.
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
        while spawn_timer.is_expired() {
            // Calculate the time elapsed since the spawn was supposed to happen.
            // This is used to adjust the spawn position for smooth, consistent spacing.
            let time_t = -spawn_timer.remaining;

            // Choose a random lane for the obstacle.
            let lane_index = rand::random_range(0..NUM_LANES);
            let lane_x = LANE_LOCATIONS[lane_index];

            let model_handle = cached.models.get(&ObstacleModel::Rail0).unwrap();

            // Spawn the obstacle entity.
            // The Z position is adjusted by `forward_move.velocity * time_t` to compensate for any delay
            // in the timer, ensuring obstacles appear at a consistent distance from the player
            // regardless of frame rate fluctuations.
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
    mut fuel: ResMut<TrainFuel>,
    mut invincible_timer: ResMut<InvincibleTimer>,
    player_query: Query<&Collider, (With<PlayerCollider>, Without<ObstacleCollider>)>,
    obstacle_query: Query<&Collider, (With<ObstacleCollider>, Without<PlayerCollider>)>,
) {
    if invincible_timer.is_expired()
        && let Ok(player_collider) = player_query.single()
    {
        for obstacle_collider in obstacle_query.iter() {
            if player_collider.intersects(obstacle_collider) {
                info!("Collision detected!");
                fuel.saturating_sub(10.0);
                invincible_timer.reset();
                break;
            }
        }
    }
}

/// A system that updates the score UI by setting the texture atlas index for each digit.
#[allow(clippy::type_complexity)]
pub fn update_score_ui(
    score: Res<PlayScore>,
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

/// A system that animates the decorative fuel icon, making it bob up and down.
/// This adds a subtle visual flair to the UI.
pub fn update_fuel_deco(mut query: Query<&mut Node, With<FuelDeco>>, time: Res<Time>) {
    if let Ok(mut node) = query.single_mut() {
        // Use a sine wave based on the elapsed game time to create a smooth, periodic vertical motion.
        let t = time.elapsed_secs() * FUEL_DECO_CYCLE;

        // Apply the sine wave to the icon's `bottom` position.
        // The icon moves between 10% (12.5 - 2.5) and 15% (12.5 + 2.5) from the bottom of its container.
        node.bottom = Val::Percent(12.5 + 2.5 * t.sin());
    }
}

/// A system that updates the fuel gauge's width to visually represent the remaining fuel percentage.
pub fn update_fuel_gauge(mut query: Query<&mut Node, With<FuelGauge>>, fuel: Res<TrainFuel>) {
    if let Ok(mut node) = query.single_mut() {
        // Directly map the remaining fuel (which is a percentage from 0.0 to 100.0)
        // to the width of the UI node, also as a percentage.
        node.width = Val::Percent(fuel.remaining);
    }
}

/// A system that applies a visual effect to the toy trains when the player is invincible.
/// It iterates through each train car and calls a recursive helper function to apply the effect to all its child meshes.
#[allow(clippy::type_complexity)]
pub fn update_invincible_effect(
    mut set: ParamSet<(
        Query<Entity, With<ToyTrain0>>,
        Query<Entity, With<ToyTrain1>>,
        Query<Entity, With<ToyTrain2>>,
    )>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    invincible_timer: Res<InvincibleTimer>,
) {
    if let Ok(entity) = set.p0().single() {
        update_invincible_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &invincible_timer,
        );
    }

    if let Ok(entity) = set.p1().single() {
        update_invincible_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &invincible_timer,
        );
    }

    if let Ok(entity) = set.p2().single() {
        update_invincible_effect_recursive(
            entity,
            &children_query,
            &material_query,
            &mut materials,
            &invincible_timer,
        );
    }
}

/// A recursive helper function that traverses the entity hierarchy and applies the invincibility effect.
/// It changes the `base_color` of any `StandardMaterial` found on the entity or its children,
/// creating a flashing white effect based on the remaining invincibility time.
fn update_invincible_effect_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    invincible_timer: &Res<InvincibleTimer>,
) {
    // Check if the current entity has a material and apply the effect.
    if let Ok(handle) = material_query.get(entity)
        && let Some(material) = materials.get_mut(handle.id())
    {
        // Use a cosine wave to smoothly oscillate the color between black and white.
        let t = invincible_timer.remaining * INVINCIBLE_EFFECT_CYCLE;
        let fill = 0.5 * t.cos() + 0.5;
        material.base_color = Color::srgb(fill, fill, fill);
    }

    // Recursively call this function for all children of the current entity.
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_invincible_effect_recursive(
                child,
                children_query,
                material_query,
                materials,
                invincible_timer,
            );
        }
    }
}
