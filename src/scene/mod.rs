pub mod in_game;
pub mod loading;
pub mod pause;
pub mod prepare;
pub mod resume;
pub mod wrap_up;

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
    seq::IndexedRandom,
};

use crate::{
    asset::{model::ModelAsset, spawner::SpawnModel},
    collider::Collider,
};

// --- GAME CONSTANTS ---

/// The number of lanes available to the player.
const NUM_LANES: usize = 3;
/// The maximum lane index (0-based).
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
/// The x-coordinates for each lane.
const LANE_LOCATIONS: [f32; NUM_LANES] = [-3.0, 0.25, 3.5];
/// The starting Z-position of the player during the `Prepare` scene intro animation.
const PLAYER_MIN_Z_POS: f32 = -20.0;
/// The final Z-position of the player after the `Prepare` scene intro, which is the gameplay position.
const PLAYER_MAX_Z_POS: f32 = -7.5;

/// The duration of the UI slide-in/out animations in seconds.
const UI_ANIMATION_DURATION: f32 = 1.0;

/// The delay between player inputs in seconds, to prevent overly sensitive controls.
const INPUT_DELAY: f32 = 0.25;

/// The initial forward movement speed of the player.
const SPEED: f32 = 20.0;
/// The maximum forward movement speed the player can reach.
const MAX_SPEED: f32 = 30.0;
/// The rate at which the player's speed increases over time.
const ACCELERATION: f32 = (MAX_SPEED - SPEED) / 30.0;
/// The strength of gravity affecting the player.
const GRAVITY: f32 = -30.0;
/// The initial upward velocity of the player's jump.
const JUMP_STRENGTH: f32 = 12.5;
/// The speed at which the player changes lanes.
const LANE_CHANGE_SPEED: f32 = 5.0;

/// The distance the player must travel to gain one score point from movement.
const SCORE_DIST_CYCLE: f32 = 1.0;
/// The cycle speed of the fuel decoration's bobbing animation.
const FUEL_DECO_CYCLE: f32 = PI * 1.0;
/// The cycle speed of the flashing effect when the player is attacked.
const ATTACKED_EFFECT_CYCLE: f32 = PI * 8.0; // 8 cycles per second
/// The cycle speed for the pause title's blinking effect.
const PAUSE_TITLE_CYCLE: f32 = 1.5; // 1.5 seconds per blink cycle

/// The color of the fuel gauge's decorative border.
const FUEL_COLOR: Color = Color::srgb(48.0 / 255.0, 55.0 / 255.0, 70.0 / 255.0);
/// The color of the fuel gauge's indicator bar when fuel is plentiful (green).
const FUEL_GOOD_GAUGE_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
/// The color of the fuel gauge's indicator bar when fuel is at a medium level (yellow).
const FUEL_FAIR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.8, 0.2);
/// The color of the fuel gauge's indicator bar when fuel is low (red).
const FUEL_POOR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
/// The color of the loading bar.
const LOADING_BAR_COLOR: Color = Color::srgb(0.2, 0.8, 0.2); // Green color for the loading bar.
/// The base color of the pause button.
const PAUSE_BTN_COLOR: Color = Color::WHITE;
/// The color of the pause icon (the two vertical bars).
const PAUSE_ICON_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
/// The translucent background color for the pause screen.
const UI_PAUSE_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
/// The color of the "Resume" button in the pause menu.
const RESUME_BTN_COLOR: Color = Color::WHITE;
/// The color of the "Exit" button in the pause menu.
const EXIT_BTN_COLOR: Color = Color::srgb(250.0 / 255.0, 95.0 / 255.0, 85.0 / 255.0);

/// The rate at which fuel is consumed per second.
const FUEL_USAGE: f32 = 100.0 / 20.0;

/// The duration in seconds that the player remains in the "attacked" state.
const ATTACKED_DURATION: f32 = 3.0;

// --- Obstacle Spawning Constants ---
/// The total number of different obstacle types available to spawn.
const NUM_SPAWN_OBJECTS: usize = 3;
/// An array defining the types of objects that can be spawned.
const SPAWN_OBJECTS: [SpawnObject; NUM_SPAWN_OBJECTS] =
    [SpawnObject::Fence0, SpawnObject::Stone0, SpawnObject::Fuel];
/// The corresponding weights for each object in `SPAWN_OBJECTS`, used for weighted random selection.
const SPAWN_WEIGHTS: [u32; NUM_SPAWN_OBJECTS] = [5, 5, 3];
/// The Z-coordinate where new objects are spawned, far in front of the player.
const SPAWN_LOCATION: f32 = 100.0;
/// The Z-coordinate at which objects are despawned, far behind the player.
const DESPAWN_LOCATION: f32 = -100.0;
/// The base distance the player travels before a new object group is spawned.
const SPAWN_INTERVAL: f32 = 25.0;
/// The random range of Z-offset applied to each spawned object to vary spacing.
const SPAWN_OFFSETS: RangeInclusive<f32> = -5.0..=5.0;

/// The amount of fuel lost when hitting a fence.
const FENCE_AMOUNT: f32 = 10.0;
/// The number of predefined spawn patterns for fences.
const NUM_FENCE_LOCATIONS: usize = 7;
/// The corresponding weights for each fence spawn pattern, used for random selection.
const FENCE_WEIGHTS: [u32; NUM_FENCE_LOCATIONS] = [3, 3, 2, 3, 2, 2, 1];
/// The amount of fuel lost when hitting a stone.
const STONE_AMOUNT: f32 = 20.0;
/// The number of predefined spawn patterns for stones.
const NUM_STONE_LOCATIONS: usize = 7;
/// The corresponding weights for each stone spawn pattern, used for random selection.
const STONE_WEIGHTS: [u32; NUM_STONE_LOCATIONS] = [3, 3, 2, 3, 2, 2, 1];
/// The amount of fuel gained when collecting a fuel item.
const FUEL_AMOUNT: f32 = 30.0;

lazy_static! {
    /// A map defining the collider for each spawnable object.
    static ref OBJECT_COLLIDER: HashMap<SpawnObject, Collider> = [
        (SpawnObject::Fence0, Collider::Aabb { offset: Vec3::new(0.0, 0.5, 0.0), size: Vec3::splat(1.0) }),
        (SpawnObject::Stone0, Collider::Sphere { offset: Vec3::splat(0.0), radius: 1.0 }),
        (SpawnObject::Fuel, Collider::Aabb { offset: Vec3::new(0.0, 0.0, 0.0), size: Vec3::splat(0.5) }),
    ]
    .into_iter()
    .collect();

    /// Defines the possible lane combinations for fence obstacles. Each inner vector represents a spawn pattern.
    static ref FENCE_LOCATIONS: [Vec<f32>; NUM_FENCE_LOCATIONS] = [
        vec![LANE_LOCATIONS[0]], // Pattern 1: Lane 0
        vec![LANE_LOCATIONS[1]], // Pattern 2: Lane 1
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1]], // Pattern 3: Lanes 0, 1
        vec![LANE_LOCATIONS[2]], // Pattern 4: Lane 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[2]], // Pattern 5: Lanes 0, 2
        vec![LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 6: Lanes 1, 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 7: Lanes 0, 1, 2
    ];

    /// Defines the possible lane combinations for stone obstacles. Each inner vector represents a spawn pattern.
    static ref STONE_LOCATIONS: [Vec<f32>; 7] = [
        vec![LANE_LOCATIONS[0]], // Pattern 1: Lane 0
        vec![LANE_LOCATIONS[1]], // Pattern 2: Lane 1
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1]], // Pattern 3: Lanes 0, 1
        vec![LANE_LOCATIONS[2]], // Pattern 4: Lane 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[2]], // Pattern 5: Lanes 0, 2
        vec![LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 6: Lanes 1, 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 7: Lanes 0, 1, 2
    ];
}

// --- STATES ---

/// Defines the different states of the game, controlling which systems run.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    /// The default state, where assets are loaded and a loading screen is displayed.
    #[default]
    Loading,
    /// The state where the game is paused.
    Pause,
    /// A brief introductory scene before gameplay starts.
    Prepare,
    /// The state where the main gameplay occurs. This is the primary game loop.
    InGame,
    /// The state where the game resumes after being paused.
    Resume,
    /// A brief scene that plays when the game ends (e.g., fuel runs out).
    WrapUp,
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
#[repr(usize)]
#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, Hash)]
pub enum SpawnObject {
    #[default]
    Fence0 = 0,
    Stone0 = 1,
    Fuel = 2,
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

/// A marker component to identify different parts of the UI hierarchy.
#[derive(Component, Clone, Copy)]
pub enum UI {
    /// The root node of the score display.
    Score,
    /// The root node of the fuel gauge display.
    Fuel,
    /// The "Start" message UI.
    Start,
    /// The "Finish" message UI.
    Finish,
    /// The pause button in the in-game UI.
    PauseButton,
    /// The title text of the pause menu.
    PauseTitle,
    /// The "1" in the resume countdown.
    ResumeCount1,
    /// The "2" in the resume countdown.
    ResumeCount2,
    /// The "3" in the resume countdown.
    ResumeCount3,
    /// The "Resume" button in the pause menu.
    ResumeButton,
    /// The "Exit" button in the pause menu.
    ExitButton,
}

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

/// A marker component for the pause menu title.
#[derive(Component)]
pub struct PauseTitle;

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

impl ForwardMovement {
    /// Resets the entity's forward velocity to the initial `SPEED`.
    pub fn reset(&mut self) {
        self.velocity = SPEED;
    }
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

/// A component that makes an entity rotate continuously.
#[derive(Component)]
pub struct Rotate {
    /// The axis around which the entity will rotate.
    axis: Vec3,
    /// The rotation speed in radians per second.
    radian_per_sec: f32,
}

impl Default for Rotate {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            radian_per_sec: 120f32.to_radians(),
        }
    }
}

/// A component for animating the "Start" UI element with a fade-in and fade-out effect.
#[derive(Component)]
pub struct StartAnimation {
    /// The total duration of the animation.
    duration: f32,
    /// The time elapsed since the animation started.
    elapsed: f32,
}

impl StartAnimation {
    /// Creates a new `StartAnimation`.
    pub fn new(duration: f32) -> Self {
        #[cfg(not(feature = "no-debuging-assert"))]
        assert!(duration > 0.0);

        Self {
            duration,
            elapsed: 0.0,
        }
    }

    /// Advances the animation's timer.
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }

    /// Calculates the current color based on the elapsed time, creating a fade-in then fade-out effect.
    pub fn color(&self) -> Color {
        let half_duration = self.duration * 0.5;
        if self.elapsed <= half_duration {
            // Fade-in phase using an ease-out-cubic-like curve.
            let t = self.elapsed / half_duration;
            let alpha = (t - 1.0).powi(3) * (1.0 - t) + 1.0;
            Color::srgba(1.0, 1.0, 1.0, alpha)
        } else {
            // Fade-out phase using an ease-out-quart curve.
            let t = (self.elapsed - half_duration) / half_duration;
            let alpha = 1.0 - t * t * t * t;
            Color::srgba(1.0, 1.0, 1.0, alpha)
        }
    }

    /// Checks if the animation has finished.
    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// A component for animating the "Finish" UI element with a fade-in effect.
#[derive(Component)]
pub struct FinishAnimation {
    /// The total duration of the animation.
    duration: f32,
    /// The time elapsed since the animation started.
    elapsed: f32,
}

impl FinishAnimation {
    /// Creates a new `FinishAnimation`.
    pub fn new(duration: f32) -> Self {
        #[cfg(not(feature = "no-debuging-assert"))]
        assert!(duration > 0.0);

        Self {
            duration,
            elapsed: 0.0,
        }
    }

    /// Advances the animation's timer.
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }

    /// Calculates the current color based on the elapsed time, creating a fade-in effect.
    pub fn color(&self) -> Color {
        // Fade-in using an ease-out-cubic-like curve.
        let t = (self.elapsed / self.duration).min(1.0);
        let alpha = (t - 1.0).powi(3) * (1.0 - t) + 1.0;
        Color::srgba(1.0, 1.0, 1.0, alpha)
    }
}

/// A component that holds a handle to an animation clip.
/// This is used to trigger the animation playback once the model is loaded.
#[derive(Component)]
pub struct AnimationClipHandle(pub Handle<AnimationClip>);

// --- RESOURCES ---

/// A resource to track the player's score.
#[derive(Resource)]
pub struct PlayScore {
    /// The total accumulated score.
    accum: u32,
    /// The distance traveled, used to grant score over distance.
    distance: f32,
}

impl Default for PlayScore {
    fn default() -> Self {
        Self {
            accum: 0,
            distance: 0.0,
        }
    }
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
    /// A debug state that might grant invincibility or other effects.
    #[cfg(not(feature = "no-debuging-player"))]
    Debug,
    /// The default, normal state.
    #[default]
    Idle,
    /// The state after being hit by an obstacle. Includes a timer for how long the state lasts.
    Attacked { remaining: f32 },
}

impl PlayerState {
    /// Checks if the player is in the `Debug` state (invincible).
    #[cfg(not(feature = "no-debuging-player"))]
    pub fn is_debug(&self) -> bool {
        matches!(self, PlayerState::Debug)
    }

    /// Checks if the player is in any state that grants invincibility.
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_invincible(&self) -> bool {
        match self {
            #[cfg(not(feature = "no-debuging-player"))]
            PlayerState::Debug => true,
            _ => false,
        }
    }
}

/// A resource that manages the spawning of objects over a distance.
#[derive(Resource)]
pub struct ObjectSpawner {
    /// The distance traveled since the last object group was spawned.
    distance: f32,
    /// The next object that is scheduled to be spawned (currently unused).
    next_obj: SpawnObject,
    /// A weighted distribution for selecting the type of object to spawn (Fence, Stone, or Fuel).
    object_distr: WeightedIndex<u32>,
    /// A weighted distribution for selecting the spawn pattern for fences.
    fence_distr: WeightedIndex<u32>,
    /// A weighted distribution for selecting the spawn pattern for stones.
    stone_distr: WeightedIndex<u32>,
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
        cached: &CachedObjects,
        commands: &mut Commands,
        forward_move: &ForwardMovement,
        delta_time: f32,
    ) -> bool {
        self.distance += forward_move.velocity.abs() * delta_time;
        if self.distance >= SPAWN_INTERVAL {
            let mut rng = rand::rng();
            let selected_index = self.object_distr.sample(&mut rng);
            let selected_item = SPAWN_OBJECTS[selected_index];
            let delta = SPAWN_INTERVAL - self.distance;

            let model_handle = cached.models.get(&selected_item).unwrap();
            let collider = OBJECT_COLLIDER.get(&selected_item).cloned().unwrap();
            match selected_item {
                SpawnObject::Fence0 => {
                    let locations = &FENCE_LOCATIONS[self.fence_distr.sample(&mut rng)];
                    for lane_x in locations {
                        commands.spawn((
                            SpawnModel(model_handle.clone()),
                            Transform::from_xyz(*lane_x, 0.0, SPAWN_LOCATION + delta),
                            InGameStateEntity,
                            collider,
                            selected_item,
                        ));
                    }
                }
                SpawnObject::Stone0 => {
                    let locations = &STONE_LOCATIONS[self.stone_distr.sample(&mut rng)];
                    for lane_x in locations {
                        commands.spawn((
                            SpawnModel(model_handle.clone()),
                            Transform::from_xyz(*lane_x, 0.0, SPAWN_LOCATION + delta),
                            InGameStateEntity,
                            collider,
                            selected_item,
                        ));
                    }
                }
                SpawnObject::Fuel => {
                    let lane_x = LANE_LOCATIONS.choose(&mut rng).unwrap();
                    commands.spawn((
                        SpawnModel(model_handle.clone()),
                        Transform::from_xyz(*lane_x, 0.5, SPAWN_LOCATION + delta),
                        InGameStateEntity,
                        Rotate::default(),
                        collider,
                        selected_item,
                    ));
                }
            };

            let offset = rng.random_range(SPAWN_OFFSETS);
            self.distance -= SPAWN_INTERVAL + offset;
            self.next_obj = selected_item;

            true
        } else {
            false
        }
    }
}

impl Default for ObjectSpawner {
    fn default() -> Self {
        Self {
            distance: 0.0,
            next_obj: SpawnObject::default(),
            object_distr: WeightedIndex::new(SPAWN_WEIGHTS).unwrap(),
            fence_distr: WeightedIndex::new(FENCE_WEIGHTS).unwrap(),
            stone_distr: WeightedIndex::new(STONE_WEIGHTS).unwrap(),
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

/// A resource that tracks the elapsed time of a scene, used for timed transitions or animations.
#[derive(Resource)]
pub struct SceneTimer(pub f32);

impl SceneTimer {
    /// Advances the timer by the given `delta_time`.
    pub fn tick(&mut self, delta_time: f32) {
        self.0 += delta_time;
    }
}

impl Default for SceneTimer {
    fn default() -> Self {
        Self(0.0)
    }
}
