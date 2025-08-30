pub mod in_game;
pub mod in_game_load;

use std::collections::{HashMap, VecDeque};

// Import necessary Bevy modules.
use bevy::prelude::*;

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
/// The delay between obstacle creation in seconds.
const SPAWN_DELAY: f32 = 2.0;
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
/// The color of the fuel gauge's decorative border.
const FUEL_COLOR: Color = Color::srgb(48.0 / 255.0, 55.0 / 255.0, 70.0 / 255.0);
/// The color of the fuel gauge's indicator bar.
const FUEL_GAUGE_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
/// The color of the loading bar.
const LOADING_BAR_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
/// The rate at which fuel is consumed per second.
const FUEL_USAGE: f32 = 100.0 / 16.0;

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

/// A marker component for the player's collider entity.
#[derive(Component)]
pub struct PlayerCollider;

/// A marker component for the ground plane entities.
#[derive(Component)]
pub struct Ground;

/// A marker component for obstacle entities.
#[derive(Component)]
pub struct Obstacle;

/// A marker component for an obstacle's collider entity.
#[derive(Component)]
pub struct ObstacleCollider;

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

/// A resource to manage the player's invincibility timer after a collision.
#[derive(Resource)]
pub struct InvincibleTimer {
    remaining: f32,
}

impl InvincibleTimer {
    /// Resets the invincibility timer to its full duration (3 seconds).
    pub fn reset(&mut self) {
        self.remaining = 3.0;
    }

    /// Reduces the remaining invincibility time.
    pub fn on_advanced(&mut self, duration: f32) {
        self.remaining = (self.remaining - duration).max(0.0);
    }

    /// Checks if the invincibility has expired.
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl Default for InvincibleTimer {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// A resource to manage the spawning of obstacles.
#[derive(Resource)]
pub struct ObstacleSpawnTimer {
    remaining: f32,
}

impl ObstacleSpawnTimer {
    /// Reduces the remaining time until the next spawn.
    pub fn on_advanced(&mut self, duration: f32) {
        self.remaining -= duration;
    }

    /// Checks if it's time to spawn a new obstacle.
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl Default for ObstacleSpawnTimer {
    fn default() -> Self {
        Self {
            remaining: SPAWN_DELAY,
        }
    }
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

/// An enum to identify different obstacle models.
/// This is used as a key in the `CachedObstacles` resource HashMap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ObstacleModel {
    /// The standard rail obstacle model.
    Rail0,
}

/// A resource that caches handles to obstacle models.
#[derive(Default, Resource)]
pub struct CachedObstacles {
    models: HashMap<ObstacleModel, Handle<ModelAsset>>,
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
