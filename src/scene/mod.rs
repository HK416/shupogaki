pub mod in_game;
pub mod in_game_load;

use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    ops::RangeInclusive,
};

// Import necessary Bevy modules.
use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};

use crate::asset::model::ModelAsset;

// --- GAME CONSTANTS ---

/// The number of lanes available to the player.
const NUM_LANES: usize = 3;
/// The maximum lane index (0-based).
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
/// The x-coordinates for each lane.
const LANE_LOCATIONS: [f32; NUM_LANES] = [-3.0, 0.25, 3.5];

/// The delay between player inputs in seconds, to prevent overly sensitive controls.
const INPUT_DELAY: f32 = 0.25;

/// The forward movement speed of the player and the world.
const SPEED: f32 = 20.0;
/// The strength of gravity affecting the player.
const GRAVITY: f32 = -30.0;
/// The initial upward velocity of the player's jump.
const JUMP_STRENGTH: f32 = 12.5;
/// The lane change speed of the player.
const LANE_CHANGE_SPEED: f32 = 5.0;

/// The score cycle that determines how frequently the score increases.
const SCORE_CYCLE: u32 = 100;
/// The cycle speed of the fuel decoration's bobbing animation.
const FUEL_DECO_CYCLE: f32 = PI * 1.0;
/// The cycle speed of the flashing effect when the player is attacked.
const ATTACKED_EFFECT_CYCLE: f32 = PI * 8.0;
// const INVINCIBLE_EFFECT_CYCLE: f32 = PI * 8.0;

/// The color of the fuel gauge's decorative border.
const FUEL_COLOR: Color = Color::srgb(48.0 / 255.0, 55.0 / 255.0, 70.0 / 255.0);
/// The color of the fuel gauge's indicator bar.
const FUEL_GAUGE_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
/// The color of the loading bar.
const LOADING_BAR_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);

/// The rate at which fuel is consumed per second.
const FUEL_USAGE: f32 = 100.0 / 16.0;

/// The duration in seconds that the player remains in the "attacked" state.
const ATTACKED_DURATION: f32 = 3.0;
// const INVINCIBLE_DURATION: f32 = 6.0;

// --- Obstacle Spawning Constants ---
/// The total number of different obstacle types available to spawn.
const NUM_SPAWN_OBJECTS: usize = 3;
/// An array defining the types of objects that can be spawned.
const SPAWN_OBJECTS: [SpawnObject; NUM_SPAWN_OBJECTS] =
    [SpawnObject::Fence0, SpawnObject::Stone0, SpawnObject::Fuel];
/// The corresponding weights for each object in `SPAWN_OBJECTS`, used for weighted random selection.
const SPAWN_WEIGHTS: [u32; NUM_SPAWN_OBJECTS] = [5, 5, 4];
/// The Z-coordinate where new objects are spawned, far in front of the player.
const SPAWN_LOCATION: f32 = 100.0;
/// The Z-coordinate at which objects are despawned, far behind the player.
const DESPAWN_LOCATION: f32 = -100.0;
/// The distance the player travels before a new obstacle is spawned.
const SPAWN_INTERVAL: f32 = 25.0;
/// The random range of Z-offset applied to each spawned object to vary spacing.
const SPAWN_OFFSETS: RangeInclusive<f32> = -5.0..=5.0;

/// The amount of fuel lost when hitting a fence.
const FENCE_AMOUNT: f32 = 10.0;
/// The amount of fuel lost when hitting a stone.
const STONE_AMOUNT: f32 = 20.0;
/// The amount of fuel gained when collecting a fuel item.
const FUEL_AMOUNT: f32 = 15.0;

lazy_static! {
    /// A map defining the visual and collider offset for each spawnable object.
    static ref OBJECT_OFFSETS: HashMap<SpawnObject, Vec3> = [
        (SpawnObject::Fence0, Vec3::new(0.0, 0.5, 0.0)),
        (SpawnObject::Stone0, Vec3::new(0.0, 0.5, 0.0)),
        (SpawnObject::Fuel, Vec3::new(0.0, 0.5, 0.0)),
    ]
    .into_iter()
    .collect();
    /// A map defining the collider extents (size) for each spawnable object.
    static ref OBJECT_EXTENTS: HashMap<SpawnObject, Vec3> = [
        (SpawnObject::Fence0, Vec3::splat(1.0)),
        (SpawnObject::Stone0, Vec3::splat(1.0)),
        (SpawnObject::Fuel, Vec3::splat(0.5)),
    ]
    .into_iter()
    .collect();
}

// --- STATES ---

/// Defines the different states of the game, controlling which systems run.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    /// The default state, where assets are loaded.
    #[default]
    InGameLoading,
    /// The state where the main gameplay occurs.
    InGame,
}

// --- COMPONENTS ---

/// A marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// A marker component for the ground plane entities.
#[derive(Component)]
pub struct Ground;

/// An enum representing the different types of objects that can be spawned in the game.
/// This is used as a component to identify and differentiate game objects.
#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, Hash)]
pub enum SpawnObject {
    #[default]
    Fence0,
    Stone0,
    Fuel,
    // Aoba,
}

/// A marker component for entities that should only exist during the `InGameLoad` state.
#[derive(Component)]
pub struct InGameLoadStateEntity;

/// A marker component for entities that should only exist during the `InGame` state.
#[derive(Component)]
pub struct InGameStateEntity;

/// A marker component for the first toy train entity.
#[derive(Component)]
pub struct ToyTrain0;

/// A marker component for the second toy train entity.
#[derive(Component)]
pub struct ToyTrain1;

/// A marker component for the third toy train entity.
#[derive(Component)]
pub struct ToyTrain2;

/// A marker component for the score text UI element.
#[derive(Component)]
pub struct Score;

/// A marker component for the fuel gauge's decorative background.
#[derive(Component)]
pub struct FuelDeco;

/// A marker component for the fuel gauge's value bar.
#[derive(Component)]
pub struct FuelGauge;

/// A marker component for the 1s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace1s;

/// A marker component for the 10s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace10s;

/// A marker component for the 100s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace100s;

/// A marker component for the 1,000s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace1000s;

/// A marker component for the 10,000s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace10000s;

/// A marker component for the 100,000s place digit of the score display.
#[derive(Component)]
pub struct ScoreSpace100000s;

/// Stores the player's current lane index.
#[derive(Component)]
pub struct Lane {
    index: usize,
}

impl Default for Lane {
    fn default() -> Self {
        // Start the player in the middle lane.
        Self {
            index: NUM_LANES / 2,
        }
    }
}

/// Stores the entity's forward movement speed.
#[derive(Component)]
pub struct ForwardMovement {
    velocity: f32,
}

impl Default for ForwardMovement {
    fn default() -> Self {
        Self { velocity: SPEED }
    }
}

/// Stores the entity's vertical movement speed for jumping and gravity.
#[derive(Component)]
pub struct VerticalMovement {
    velocity: f32,
}

impl Default for VerticalMovement {
    fn default() -> Self {
        Self { velocity: 0.0 }
    }
}

/// A component that holds a handle to an animation clip.
/// This is used to trigger the animation playback once the model is loaded.
#[derive(Component)]
pub struct AnimationClipHandle(pub Handle<AnimationClip>);

// --- RESOURCES ---

/// A resource to track the player's score.
#[derive(Default, Resource)]
pub struct PlayScore {
    /// A timer that accumulates milliseconds.
    timer: u32,
    /// The total accumulated score.
    accum: u32,
}

/// A resource to manage the delay between player inputs.
#[derive(Resource)]
pub struct InputDelay {
    remaining: f32,
}

impl InputDelay {
    /// Reduces the remaining delay time.
    pub fn on_advanced(&mut self, duration: f32) {
        self.remaining = (self.remaining - duration).max(0.0)
    }

    /// Checks if the input delay has expired.
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Resets the input delay to its initial value.
    pub fn reset(&mut self) {
        self.remaining = INPUT_DELAY;
    }
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// Defines the player's current state, which can affect gameplay and visual effects.
#[derive(Clone, Copy, Default, Resource)]
pub enum PlayerState {
    #[cfg(not(feature = "no-debuging-player"))]
    Debug,
    /// The default, normal state.
    #[default]
    Idle,
    /// The state after being hit by an obstacle. Includes a timer for how long the state lasts.
    Attacked { remaining: f32 },
    // Invincible {
    //     remaining: f32,
    // },
}

impl PlayerState {
    #[cfg(not(feature = "no-debuging-player"))]
    pub fn is_debug(&self) -> bool {
        matches!(self, PlayerState::Debug)
    }

    #[allow(clippy::match_like_matches_macro)]
    pub fn is_invincible(&self) -> bool {
        match self {
            #[cfg(not(feature = "no-debuging-player"))]
            PlayerState::Debug => true,
            // PlayerState::Invincible { .. } => true,
            _ => false,
        }
    }
}

/// A resource that manages the spawning of objects over a distance.
#[derive(Resource)]
pub struct ObjectSpawner {
    /// The distance traveled since the last object was spawned.
    distance: f32,
    /// The next object that is scheduled to be spawned.
    next_obj: SpawnObject,
}

impl ObjectSpawner {
    /// Advances the spawner based on the distance the player has traveled.
    ///
    /// If the traveled distance exceeds the `SPAWN_INTERVAL`, this method determines
    /// that a new object should be spawned. It returns the type of object to spawn
    /// and a `delta` value, which is the small amount of Z-offset needed to ensure
    /// the object spawns at the correct position relative to the player, even if the
    /// frame rate varies. It then schedules the next object to be spawned.
    pub fn on_advanced(
        &mut self,
        forward_move: &ForwardMovement,
        delta_time: f32,
    ) -> Option<(SpawnObject, f32)> {
        self.distance += forward_move.velocity.abs() * delta_time;
        if self.distance >= SPAWN_INTERVAL {
            let mut rng = rand::rng();
            let distr = WeightedIndex::new(SPAWN_WEIGHTS).unwrap();
            let selected_index = distr.sample(&mut rng);
            let selected_item = SPAWN_OBJECTS[selected_index];

            let obj = self.next_obj;
            let delta = SPAWN_INTERVAL - self.distance;

            let offset = rng.random_range(SPAWN_OFFSETS);
            self.distance -= SPAWN_INTERVAL + offset;
            self.next_obj = selected_item;

            Some((obj, delta))
        } else {
            None
        }
    }
}

impl Default for ObjectSpawner {
    fn default() -> Self {
        Self {
            distance: 0.0,
            next_obj: SpawnObject::default(),
        }
    }
}

/// A resource that caches handles to all spawnable object models.
#[derive(Default, Resource)]
pub struct CachedObjects {
    models: HashMap<SpawnObject, Handle<ModelAsset>>,
}

/// An enum to identify different ground models.
/// This is used as a key in the `CachedGrounds` resource HashMap to retrieve the correct model handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroundModel {
    /// The standard ground plane model.
    Plane0,
}

/// A resource that caches handles to ground models to avoid reloading them.
#[derive(Default, Resource)]
pub struct CachedGrounds {
    models: HashMap<GroundModel, Handle<ModelAsset>>,
}

/// A resource that holds the transforms of ground entities that have moved off-screen
/// and are ready to be reused.
#[derive(Default, Resource)]
pub struct RetiredGrounds {
    transforms: VecDeque<Transform>,
}

/// A resource to manage the player's fuel level.
#[derive(Resource)]
pub struct TrainFuel {
    remaining: f32,
}

impl TrainFuel {
    /// Adds a specified amount to the fuel, capping at 100.0.
    pub fn add(&mut self, amount: f32) {
        self.remaining = (self.remaining + amount).min(100.0);
    }

    /// Subtracts a specified amount from the fuel, ensuring it doesn't go below 0.0.
    pub fn saturating_sub(&mut self, amount: f32) {
        self.remaining = (self.remaining - amount).max(0.0);
    }

    /// Checks if the fuel is empty.
    pub fn is_empty(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl Default for TrainFuel {
    /// Initializes fuel to the maximum value of 100.0.
    fn default() -> Self {
        Self { remaining: 100.0 }
    }
}
