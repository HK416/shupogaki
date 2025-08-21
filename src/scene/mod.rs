pub mod in_game;
pub mod in_game_load;

use std::collections::{HashMap, VecDeque};

// Import necessary Bevy modules.
use bevy::prelude::*;

// --- GAME CONSTANTS ---

/// The number of lanes available to the player.
const NUM_LANES: usize = 3;
/// The maximum lane index (0-based).
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
/// The x-coordinates for each lane.
const LANE_LOCATIONS: [f32; NUM_LANES] = [-3.0, 0.0, 3.0];
/// The delay between player inputs in seconds.
const INPUT_DELAY: f32 = 0.25;
/// The delay between obstacle creation in seconds.
const SPAWN_DELAY: f32 = 2.0;
/// The forward movement speed of the player and the world.
const SPEED: f32 = 20.0;
/// The strength of gravity affecting the player.
const GRAVITY: f32 = -30.0;
/// The initial upward velocity of the player's jump.
const JUMP_STRENGTH: f32 = 12.5;
// The lane change speed of the player.
const LANE_CHANGE_SPEED: f32 = 5.0;

// --- STATES ---

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    InGameLoading,
    InGame,
}

// --- COMPONENTS ---

/// A marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// A marker component for the ground plane entities.
#[derive(Component)]
pub struct Ground;

/// A marker component for obstacle entities.
#[derive(Component)]
pub struct Obstacle;

/// A marker component for in_game_load state entities.
#[derive(Component)]
pub struct InGameLoadStateEntity;

/// A marker component for in_game state entities.
#[derive(Component)]
pub struct InGameStateEntity;

/// A marker component for first toy train entity.
#[derive(Component)]
pub struct ToyTrain0;

/// A marker component for seconds toy train entity.
#[derive(Component)]
pub struct ToyTrain1;

/// A marker component for third toy train entity.
#[derive(Component)]
pub struct ToyTrain2;

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

/// Stores the player's horizontal movement speed.
#[derive(Component)]
pub struct ForwardMovement {
    velocity: f32,
}

impl Default for ForwardMovement {
    fn default() -> Self {
        Self { velocity: SPEED }
    }
}

/// Stores the player's vertical movement speed for jumping and gravity.
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

/// A resource to manage the delay between player inputs.
#[derive(Resource)]
pub struct InputDelay {
    remaining: f32,
}

impl Default for InputDelay {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

/// A resource to manage the spawning of obstacles.
#[derive(Resource)]
pub struct ObstacleSpawnTimer {
    remaining: f32,
}

impl Default for ObstacleSpawnTimer {
    fn default() -> Self {
        Self {
            remaining: SPAWN_DELAY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroundModel {
    Test0,
    Test1,
    Test2,
}

#[derive(Default, Resource)]
pub struct GroundModels {
    meshes: HashMap<GroundModel, Handle<Mesh>>,
    materials: HashMap<GroundModel, Handle<StandardMaterial>>,
}

#[derive(Default, Resource)]
pub struct RetiredGrounds {
    transforms: VecDeque<Transform>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ObstacleModel {
    Test,
}

#[derive(Default, Resource)]
pub struct ObstacleModels {
    meshes: HashMap<ObstacleModel, Handle<Mesh>>,
    materials: HashMap<ObstacleModel, Handle<StandardMaterial>>,
}
