use std::{f32::consts::PI, ops::RangeInclusive};

use bevy::{platform::collections::HashMap, prelude::*};
use lazy_static::lazy_static;
use rand::distr::weighted::WeightedIndex;

use crate::collider::Collider;

use super::*;

#[cfg(target_arch = "wasm32")]
pub const HIGH_SCORE_KEY: &str = "high_score";

#[cfg(target_arch = "wasm32")]
pub const SYSTEM_VOLUME_KEY: &str = "system_volume";

pub const NUM_LANES: usize = 3;
pub const MAX_LANE_INDEX: usize = NUM_LANES - 1;
pub const LANE_POSITIONS: [f32; NUM_LANES] = [-3.0, 0.25, 3.5];

pub const PLAYER_MIN_Z_POS: f32 = -20.0;
pub const PLAYER_MAX_Z_POS: f32 = -7.5;

pub const FUEL_DECO_CYCLE: f32 = PI * 1.0;
pub const ATTACKED_EFFECT_CYCLE: f32 = PI * 8.0;
pub const MIN_INVINCIBLE_EFFECT_CYCLE: f32 = PI * 4.0;
pub const MAX_INVINCIBLE_EFFECT_CYCLE: f32 = PI * 8.0;
pub const PAUSE_TITLE_CYCLE: f32 = 1.5;

pub const SCORE_LIMITS: u32 = 999_999;
pub const FUEL_LIMITS: f32 = 100.0;
pub const INPUT_DELAY_TIME: f32 = 0.25;
pub const POINT_PER_DIST: f32 = 1.0;

pub const MIN_PLAYER_SPEED: f32 = 20.0;
pub const MAX_PLAYER_SPEED: f32 = 30.0;
pub const LANE_SWITCH_SPEED: f32 = 5.0;
pub const INVINCIBLE_SPEED: f32 = 2.0 * MAX_PLAYER_SPEED;
pub const ACCELERATION: f32 = (MAX_PLAYER_SPEED - MIN_PLAYER_SPEED) / 30.0;
pub const JUMP_STRENGTH: f32 = 12.5;
pub const GRAVITY: f32 = -30.0;
pub const FUEL_USAGE: f32 = 100.0 / 20.0;

pub const ATTACKED_DURATION: f32 = 3.0;
pub const INVINCIBLE_DURATION: f32 = 8.0;
pub const PREPARE_ANIM_DURATION: f32 = 1.0;
pub const FINISH_ANIM_DURATION: f32 = 1.0;

pub const DESPAWN_POSITION: f32 = -100.0;
pub const SPAWN_POSITION: f32 = 100.0;
pub const GROUND_SPAWN_INTERVAL: f32 = 30.0;

pub const NUM_OBJECTS: usize = 5;
pub const OBJECT_SPAWN_INTERVAL: f32 = 25.0;
pub const OBJECT_SPAWN_OFFSET: RangeInclusive<f32> = -5.0..=5.0;
pub const OBJECT_LIST: [Object; NUM_OBJECTS] = [
    Object::Barricade,
    Object::Stone,
    Object::Fuel,
    Object::Bell,
    Object::Aoba,
];

pub const NUM_BARRICADE_POSITIONS: usize = 7;
pub const NUM_STONE_POSITIONS: usize = 7;
pub const NUM_FUEL_POSITIONS: usize = 3;
pub const NUM_BELL_POSITIONS: usize = 3;
pub const NUM_AOBA_POSITIONS: usize = 3;

pub const BARRICADE_DAMAGE: f32 = 20.0;
pub const STONE_DAMAGE: f32 = 30.0;
pub const FUEL_HEALING: f32 = 30.0;
pub const BELL_POINT: u32 = 500;

lazy_static! {
    pub static ref OBJECT_MODELS: HashMap<Object, &'static str> = {
        let map: HashMap<_, _> = [
            (Object::Barricade, MODEL_PATH_BARRICADE),
            (Object::Stone, MODEL_PATH_STONE),
            (Object::Fuel, MODEL_PATH_FUEL),
            (Object::Bell, MODEL_PATH_DOOR_BELL),
            (Object::Aoba, MODEL_PATH_AOBA),
        ]
        .into_iter()
        .collect();

        assert!(map.len() == NUM_OBJECTS);
        map
    };
    pub static ref OBJECT_COLLIDER: HashMap<Object, Collider> = {
        let map: HashMap<_, _> = [
            (
                Object::Barricade,
                Collider::Aabb {
                    offset: Vec3::new(0.0, 0.5, 0.0),
                    size: Vec3::splat(1.0),
                },
            ),
            (
                Object::Stone,
                Collider::Sphere {
                    offset: Vec3::splat(0.0),
                    radius: 1.0,
                },
            ),
            (
                Object::Fuel,
                Collider::Aabb {
                    offset: Vec3::new(0.0, 0.0, 0.0),
                    size: Vec3::splat(0.5),
                },
            ),
            (
                Object::Bell,
                Collider::Aabb {
                    offset: Vec3::new(0.0, 0.0, 0.0),
                    size: Vec3::splat(0.5),
                },
            ),
            (
                Object::Aoba,
                Collider::Aabb {
                    offset: Vec3::new(0.0, 0.5, 0.0),
                    size: Vec3::new(0.5, 1.0, 0.5),
                },
            ),
        ]
        .into_iter()
        .collect();

        assert!(map.len() == NUM_OBJECTS);
        map
    };
}

lazy_static! {
    pub static ref SOUND_DAMAGED_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_SOUND_VO_DAMAGED] = [5, 5, 5, 5, 1, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
    pub static ref SPAWN_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_OBJECTS] = [400, 300, 200, 95, 5];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
    pub static ref BARRICADE_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_BARRICADE_POSITIONS] = [3, 3, 2, 3, 2, 2, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
    pub static ref STONE_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_STONE_POSITIONS] = [3, 3, 2, 3, 2, 2, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
}

lazy_static! {
    #[rustfmt::skip]
    pub static ref BARRICADE_POSITION_INDICES: [Vec<usize>; NUM_BARRICADE_POSITIONS] = [
        vec![0], vec![1], vec![0, 1], vec![2], vec![0, 2], vec![1, 2], vec![0, 1, 2],
    ];

    #[rustfmt::skip]
    pub static ref STONE_POSITION_INDICES: [Vec<usize>; NUM_STONE_POSITIONS] = [
        vec![0], vec![1], vec![0, 1], vec![2], vec![0, 2], vec![1, 2], vec![0, 1, 2],
    ];
}

pub const FUEL_POSITION_INDICES: [usize; NUM_FUEL_POSITIONS] = [0, 1, 2];
pub const BELL_POSITION_INDICES: [usize; NUM_BELL_POSITIONS] = [0, 1, 2];
pub const AOBA_POSITION_INDICES: [usize; NUM_AOBA_POSITIONS] = [0, 1, 2];

pub const LANGUAGE_BTN_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const SLIDER_RAIL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const SLIDER_HANDLE_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
pub const CLEAR_COLOR: Color = Color::srgb(0.48627, 0.81568, 1.0);
pub const LOADING_BAR_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
pub const RESUME_BTN_COLOR: Color = Color::WHITE;
pub const OPTION_BTN_COLOR: Color = Color::WHITE;
pub const RESTART_BTN_COLOR: Color = Color::WHITE;
pub const EXIT_BTN_COLOR: Color = Color::srgb(0.98039, 0.37254, 0.33333);
pub const BACK_BTN_COLOR: Color = Color::srgb(0.98039, 0.37254, 0.33333);
pub const PAUSE_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
pub const PAUSE_BTN_COLOR: Color = Color::WHITE;
pub const PAUSE_ICON_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
pub const FUEL_COLOR: Color = Color::srgb(0.18823, 0.21568, 0.27450);
pub const FUEL_GOOD_GAUGE_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
pub const FUEL_FAIR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.8, 0.2);
pub const FUEL_POOR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
