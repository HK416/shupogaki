use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};
use rand::{Rng, distr::Distribution, seq::IndexedRandom};

use crate::asset::{animation::AnimationClipHandle, spawner::SpawnModel};

use super::*;

#[derive(Default, Resource)]
pub struct HighScore(pub u32);

#[derive(Default, Resource, Deref, DerefMut)]
pub struct RetryCounter(pub u32);

#[derive(Resource)]
pub struct SceneTimer {
    elapsed_time: f32,
}

impl SceneTimer {
    pub fn elapsed_sec(&self) -> f32 {
        self.elapsed_time
    }

    pub fn tick(&mut self, elapsed: f32) {
        self.elapsed_time += elapsed;
    }

    pub fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }
}

impl Default for SceneTimer {
    fn default() -> Self {
        Self { elapsed_time: 0.0 }
    }
}

#[derive(Default, Resource)]
pub struct PlayTime {
    play_time_ms: u128,
}

impl PlayTime {
    pub fn tick(&mut self, time: &Time) {
        self.play_time_ms = self.play_time_ms.saturating_add(time.delta().as_millis());
    }

    pub fn millis(&self) -> u128 {
        self.play_time_ms
    }
}

#[derive(Default, Resource)]
pub struct Attacked {
    count: u32,
}

impl Attacked {
    pub fn add(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}

#[derive(Resource)]
pub struct InputDelay {
    remaining: f32,
}

impl InputDelay {
    pub fn on_advanced(&mut self, elapsed: f32) {
        self.remaining = (self.remaining - elapsed).max(0.0);
    }

    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    pub fn reset(&mut self) {
        self.remaining = INPUT_DELAY_TIME;
    }
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

#[derive(Resource)]
pub struct CurrentScore {
    point: u32,
    distance: f32,
}

impl CurrentScore {
    pub fn get(&self) -> u32 {
        self.point
    }

    pub fn inc(&mut self, amount: u32) {
        self.point = (self.point + amount).min(SCORE_LIMITS);
    }

    pub fn on_advanced(&mut self, forward_move: &ForwardMovement, elapsed: f32) {
        self.distance += forward_move.get() * elapsed;
        let amount = (self.distance / POINT_PER_DIST).floor() as u32;
        self.point = (self.point + amount).min(SCORE_LIMITS);
        self.distance %= POINT_PER_DIST;
    }
}

impl Default for CurrentScore {
    fn default() -> Self {
        Self {
            point: 0,
            distance: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct TrainFuel {
    remaining: f32,
}

impl TrainFuel {
    pub fn get(&self) -> f32 {
        self.remaining
    }

    pub fn set(&mut self, amount: f32) {
        self.remaining = amount.min(FUEL_LIMITS);
    }

    pub fn inc(&mut self, amount: f32) {
        self.remaining = (self.remaining + amount).min(FUEL_LIMITS);
    }

    pub fn dec(&mut self, amount: f32) {
        self.remaining = (self.remaining - amount).max(0.0);
    }

    pub fn is_empty(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl Default for TrainFuel {
    fn default() -> Self {
        Self {
            remaining: FUEL_LIMITS,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Resource)]
pub struct IsPlayerJumping {
    prev: bool,
    curr: bool,
}

impl IsPlayerJumping {
    pub fn jump(&mut self) {
        self.prev = self.curr;
        self.curr = true;
    }

    pub fn reset(&mut self) {
        self.prev = self.curr;
        self.curr = false;
    }

    pub fn get(&self) -> bool {
        self.curr
    }

    pub fn changed(&self) -> bool {
        self.prev != self.curr
    }
}

#[derive(Debug, Default, Clone, Copy, Resource)]
pub enum CurrentState {
    #[cfg(not(feature = "no-debuging-player"))]
    Debug,
    #[default]
    Idle,
    Attacked {
        remaining: f32,
    },
    Invincible {
        remaining: f32,
    },
}

impl CurrentState {
    /// Checks if the player is in the `Debug` state (invincible).
    #[cfg(not(feature = "no-debuging-player"))]
    pub fn is_debug(&self) -> bool {
        matches!(self, CurrentState::Debug)
    }

    /// Checks if the player is in any state that grants invincibility.
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_invincible(&self) -> bool {
        match self {
            #[cfg(not(feature = "no-debuging-player"))]
            CurrentState::Debug => true,
            CurrentState::Invincible { .. } => true,
            _ => false,
        }
    }
}

#[derive(Default, Resource)]
pub struct LoadingEntities {
    pub handles: Vec<Entity>,
}

#[derive(Default, Resource)]
pub struct SystemAssets {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Default, Resource)]
pub struct TitleAssets {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Default, Resource)]
pub struct InGameAssets {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Resource)]
pub struct RetiredGrounds {
    entities: VecDeque<Entity>,
}

impl RetiredGrounds {
    pub fn push(&mut self, entity: Entity) {
        self.entities.push_back(entity);
    }

    pub fn pop(&mut self) -> Option<Entity> {
        self.entities.pop_front()
    }
}

impl Default for RetiredGrounds {
    fn default() -> Self {
        Self {
            entities: VecDeque::with_capacity(8),
        }
    }
}

#[derive(Resource)]
pub struct ObjectSpawner {
    distance: f32,
    next_obj: Object,
    retired: HashMap<Object, VecDeque<Entity>>,
}

impl ObjectSpawner {
    pub fn on_advanced(
        &mut self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        forward_move: &ForwardMovement,
        elapsed: f32,
    ) {
        let mut rng = rand::rng();
        self.distance += forward_move.get() * elapsed;
        while self.distance >= OBJECT_SPAWN_INTERVAL {
            let path = OBJECT_MODELS.get(&self.next_obj).cloned().unwrap();
            let collider = OBJECT_COLLIDER.get(&self.next_obj).cloned().unwrap();
            let model = asset_server.load(path);
            let delta = OBJECT_SPAWN_INTERVAL - self.distance;

            match self.next_obj {
                Object::Barricade => {
                    let index = BARRICADE_WEIGHTS.sample(&mut rng);
                    let indices = &BARRICADE_POSITION_INDICES[index];
                    for &lane_index in indices {
                        let recycle = self
                            .retired
                            .get_mut(&self.next_obj)
                            .and_then(|entities| entities.pop_front());

                        match recycle {
                            Some(entity) => {
                                info!("Recycle Barricade entity");
                                commands.entity(entity).insert((
                                    Lane::new(lane_index),
                                    Transform::from_xyz(
                                        LANE_POSITIONS[lane_index],
                                        0.0,
                                        SPAWN_POSITION + delta,
                                    ),
                                ));
                            }
                            None => {
                                info!("Spawn Barricade entity");
                                commands.spawn((
                                    SpawnModel(model.clone()),
                                    Lane::new(lane_index),
                                    Transform::from_xyz(
                                        LANE_POSITIONS[lane_index],
                                        0.0,
                                        SPAWN_POSITION + delta,
                                    ),
                                    InGameStateRoot,
                                    self.next_obj,
                                    collider,
                                ));
                            }
                        }
                    }
                }
                Object::Stone => {
                    let index = STONE_WEIGHTS.sample(&mut rng);
                    let indices = &STONE_POSITION_INDICES[index];
                    for &lane_index in indices {
                        let recycle = self
                            .retired
                            .get_mut(&self.next_obj)
                            .and_then(|entities| entities.pop_front());

                        match recycle {
                            Some(entity) => {
                                info!("Recycle Stone entity");
                                commands.entity(entity).insert((
                                    Lane::new(lane_index),
                                    Transform::from_xyz(
                                        LANE_POSITIONS[lane_index],
                                        0.0,
                                        SPAWN_POSITION + delta,
                                    ),
                                ));
                            }
                            None => {
                                info!("Spawn Stone entity");
                                commands.spawn((
                                    SpawnModel(model.clone()),
                                    Lane::new(lane_index),
                                    Transform::from_xyz(
                                        LANE_POSITIONS[lane_index],
                                        0.0,
                                        SPAWN_POSITION + delta,
                                    ),
                                    InGameStateRoot,
                                    self.next_obj,
                                    collider,
                                ));
                            }
                        }
                    }
                }
                Object::Fuel => {
                    let lane_index = FUEL_POSITION_INDICES.choose(&mut rng).copied().unwrap();
                    let recycle = self
                        .retired
                        .get_mut(&self.next_obj)
                        .and_then(|entities| entities.pop_front());

                    match recycle {
                        Some(entity) => {
                            info!("Recycle Fuel entity");
                            commands.entity(entity).insert((
                                Lane::new(lane_index),
                                Transform::from_xyz(
                                    LANE_POSITIONS[lane_index],
                                    0.5,
                                    SPAWN_POSITION + delta,
                                ),
                            ));
                        }
                        None => {
                            info!("Spawn Fuel entity");
                            commands.spawn((
                                SpawnModel(model.clone()),
                                Lane::new(lane_index),
                                Transform::from_xyz(
                                    LANE_POSITIONS[lane_index],
                                    0.5,
                                    SPAWN_POSITION + delta,
                                ),
                                RotateAnimation {
                                    axis: Vec3::Y,
                                    radian_per_sec: 120f32.to_radians(),
                                },
                                InGameStateRoot,
                                self.next_obj,
                                collider,
                            ));
                        }
                    }
                }
                Object::Bell => {
                    let lane_index = BELL_POSITION_INDICES.choose(&mut rng).copied().unwrap();
                    let recycle = self
                        .retired
                        .get_mut(&self.next_obj)
                        .and_then(|entities| entities.pop_front());

                    match recycle {
                        Some(entity) => {
                            info!("Recycle Bell entity");
                            commands.entity(entity).insert((
                                Lane::new(lane_index),
                                Transform::from_xyz(
                                    LANE_POSITIONS[lane_index],
                                    0.5,
                                    SPAWN_POSITION + delta,
                                ),
                            ));
                        }
                        None => {
                            info!("Spawn Bell entity");
                            commands.spawn((
                                SpawnModel(model.clone()),
                                Lane::new(lane_index),
                                Transform::from_xyz(
                                    LANE_POSITIONS[lane_index],
                                    0.5,
                                    SPAWN_POSITION + delta,
                                ),
                                RotateAnimation {
                                    axis: Vec3::Y,
                                    radian_per_sec: 120f32.to_radians(),
                                },
                                InGameStateRoot,
                                self.next_obj,
                                collider,
                            ));
                        }
                    }
                }
                Object::Aoba => {
                    info!("Spawn Aoba entity");
                    let lane_index = AOBA_POSITION_INDICES.choose(&mut rng).copied().unwrap();
                    commands
                        .spawn((
                            Lane::new(lane_index),
                            Transform::from_xyz(
                                LANE_POSITIONS[lane_index],
                                0.0,
                                SPAWN_POSITION + delta,
                            ),
                            InGameStateRoot,
                            self.next_obj,
                            collider,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                SpawnModel(model.clone()),
                                AnimationClipHandle(asset_server.load(ANIM_PATH_AOBA)),
                                Transform::IDENTITY.looking_to(*in_game::IN_GAME_AOBA_DIR, Vec3::Y),
                                InGameStateEntity,
                            ));

                            let direction = *in_game::IN_GAME_AOBA_DIR;
                            let translation = direction * 2.0 + Vec3::Y * 10.0;
                            parent.spawn((
                                SpawnModel(asset_server.load(MODEL_PATH_GLOW)),
                                Transform::from_translation(translation)
                                    .with_scale((3.0, 20.0, 3.0).into())
                                    .looking_to(direction, Vec3::Y),
                                GlowRoot,
                            ));
                        });
                }
            }

            let offset = rng.random_range(OBJECT_SPAWN_OFFSET);
            let index = SPAWN_WEIGHTS.sample(&mut rng);
            let next_obj = OBJECT_LIST[index];

            self.distance -= OBJECT_SPAWN_INTERVAL + offset;
            self.next_obj = next_obj;
        }
    }

    pub fn drain(&mut self, commands: &mut Commands, entity: Entity, obj: Object) {
        if !matches!(obj, Object::Aoba) {
            commands.entity(entity).despawn();
        } else {
            self.retired
                .entry(obj)
                .and_modify(|entities| {
                    entities.push_back(entity);
                })
                .or_insert(VecDeque::from_iter([entity]));
            commands.entity(entity).remove::<Object>();
        }
    }
}

impl Default for ObjectSpawner {
    fn default() -> Self {
        Self {
            distance: 0.0,
            next_obj: Object::default(),
            retired: HashMap::default(),
        }
    }
}
