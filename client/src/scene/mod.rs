mod in_game;
mod initialize;
mod option;
mod pause;
mod result;
mod setup;
mod title;

use std::{
    collections::VecDeque,
    f32::consts::{PI, TAU},
    hash::Hash,
    ops::RangeInclusive,
};

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    platform::collections::HashMap,
    prelude::*,
    window::WindowResized,
};
use const_format::concatcp;
use lazy_static::lazy_static;
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
    seq::IndexedRandom,
};

#[cfg(target_arch = "wasm32")]
use web_sys::{Storage, window};

use crate::{
    asset::{animation::AnimationClipHandle, spawner::SpawnModel},
    collider::Collider,
};

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(setup::StatePlugin)
            .add_plugins(initialize::StatePlugin)
            .add_plugins(option::StatePlugin)
            .add_plugins(pause::StatePlugin)
            .add_plugins(title::StatePlugin)
            .add_plugins(in_game::StatePlugin)
            .add_plugins(result::StatePlugin)
            .add_systems(Update, (initialize_font_size, update_font_size));
    }
}

// --- ASSET PATH ---
#[rustfmt::skip] const QUERY: &str = "?";
#[rustfmt::skip] const VERSION: &str = concat!("v=", env!("CARGO_PKG_VERSION_MINOR"));

#[rustfmt::skip] const LOCALE_PATH_EN: &str = concatcp!("locale/en.json", QUERY, VERSION);
#[rustfmt::skip] const LOCALE_PATH_JA: &str = concatcp!("locale/ja.json", QUERY, VERSION);
#[rustfmt::skip] const LOCALE_PATH_KO: &str = concatcp!("locale/ko.json", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NOTOSANS_BOLD: &str = concatcp!("fonts/NotoSans-Bold.otf", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_START: &str = concatcp!("fonts/ImgFont_Start.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_FINISH: &str = concatcp!("fonts/ImgFont_Finish.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_PAUSE: &str = concatcp!("fonts/ImgFont_Pause.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NEW: &str = concatcp!("fonts/ImgFont_New.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NUM_1: &str = concatcp!("fonts/ImgFont_1.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NUM_2: &str = concatcp!("fonts/ImgFont_2.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NUM_3: &str = concatcp!("fonts/ImgFont_3.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_TIME: &str = concatcp!("fonts/ImgFont_Time.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_SCORE: &str = concatcp!("fonts/ImgFont_Score.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_BEST: &str = concatcp!("fonts/ImgFont_Best.sprite", QUERY, VERSION);
#[rustfmt::skip] const FONT_PATH_NUMBER: &str = concatcp!("fonts/ImgFont_Number.sprite", QUERY, VERSION);
#[rustfmt::skip] const ATLAS_PATH_NUMBER: &str = concatcp!("fonts/ImgFont_Number.atlas", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_HIKARI_TITLE: &str = concatcp!("sounds/Hikari_Title.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_NOZOMI_TITLE: &str = concatcp!("sounds/Nozomi_Title.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_BACKGROUND: &str = concatcp!("sounds/Theme_253_Game.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_UI_START: &str = concatcp!("sounds/UI_Start.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_UI_FINISH: &str = concatcp!("sounds/UI_Finish.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_UI_BUTTON_BACK: &str = concatcp!("sounds/UI_Button_Back.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_UI_BUTTON_TOUCH: &str = concatcp!("sounds/UI_Button_Touch.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_UI_LOADING: &str = concatcp!("sounds/UI_Loading.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_DOOR_BELL: &str = concatcp!("sounds/SFX_DoorBell.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_START: &str = concatcp!("sounds/SFX_Train_Start.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_LOOP_1: &str = concatcp!("sounds/SFX_Train_Loop_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_LOOP_2: &str = concatcp!("sounds/SFX_Train_Loop_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_END: &str = concatcp!("sounds/SFX_Train_End.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_LANDING: &str = concatcp!("sounds/SFX_Train_Landing.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_SFX_TRAIN_INVINCIBLE: &str = concatcp!("sounds/SFX_Train_Invincible.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_START_00: &str = concatcp!("sounds/VO_Start_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_START_01: &str = concatcp!("sounds/VO_Start_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_START_02: &str = concatcp!("sounds/VO_Start_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_START_03: &str = concatcp!("sounds/VO_Start_03.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_00: &str = concatcp!("sounds/VO_Damaged_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_01: &str = concatcp!("sounds/VO_Damaged_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_02: &str = concatcp!("sounds/VO_Damaged_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_03: &str = concatcp!("sounds/VO_Damaged_03.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_04: &str = concatcp!("sounds/VO_Damaged_04.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_DAMAGED_05: &str = concatcp!("sounds/VO_Damaged_05.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_HEALING_00: &str = concatcp!("sounds/VO_Healing_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_HEALING_01: &str = concatcp!("sounds/VO_Healing_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_HEALING_02: &str = concatcp!("sounds/VO_Healing_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_HEALING_03: &str = concatcp!("sounds/VO_Healing_03.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_INVINCIBLE_00: &str = concatcp!("sounds/VO_Invincible_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_INVINCIBLE_01: &str = concatcp!("sounds/VO_Invincible_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_INVINCIBLE_02: &str = concatcp!("sounds/VO_Invincible_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_INVINCIBLE_03: &str = concatcp!("sounds/VO_Invincible_03.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_RESULT_00: &str = concatcp!("sounds/VO_Result_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_RESULT_01: &str = concatcp!("sounds/VO_Result_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_RESULT_02: &str = concatcp!("sounds/VO_Result_02.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_RESULT_03: &str = concatcp!("sounds/VO_Result_03.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_AOBA_00: &str = concatcp!("sounds/VO_Aoba_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_AOBA_01: &str = concatcp!("sounds/VO_Aoba_01.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_AOBA_HIT_00: &str = concatcp!("sounds/VO_Aoba_Hit_00.sound", QUERY, VERSION);
#[rustfmt::skip] const SOUND_PATH_VO_AOBA_HIT_01: &str = concatcp!("sounds/VO_Aoba_Hit_01.sound", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_AOBA: &str = concatcp!("animations/Aoba.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_HIKARI_CAFE_IDLE: &str = concatcp!("animations/Hikari_Cafe_Idle.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_HIKARI_IN_GAME: &str = concatcp!("animations/Hikari_InGame.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_HIKARI_VICTORY_START: &str = concatcp!("animations/Hikari_Victory_Start_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_HIKARI_VICTORY_END: &str = concatcp!("animations/Hikari_Victory_End_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_NOZOMI_CAFE_IDLE: &str = concatcp!("animations/Nozomi_Cafe_Idle.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_NOZOMI_IN_GAME: &str = concatcp!("animations/Nozomi_InGame.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_NOZOMI_VICTORY_START: &str = concatcp!("animations/Nozomi_Victory_Start_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] const ANIM_PATH_NOZOMI_VICTORY_END: &str = concatcp!("animations/Nozomi_Victory_End_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_PLANE_0: &str = concatcp!("models/Plane_0.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_PLANE_999: &str = concatcp!("models/Plane_999.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_TOY_TRAIN_00: &str = concatcp!("models/ToyTrain00.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_TOY_TRAIN_01: &str = concatcp!("models/ToyTrain01.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_TOY_TRAIN_02: &str = concatcp!("models/ToyTrain02.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_BARRICADE: &str = concatcp!("models/Barricade.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_STONE: &str = concatcp!("models/Stone.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_FUEL: &str = concatcp!("models/Fuel.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_DOOR_BELL: &str = concatcp!("models/DoorBell.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_AOBA: &str = concatcp!("models/Aoba.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_GLOW: &str = concatcp!("models/Glow.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_HIKARI: &str = concatcp!("models/Hikari.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const MODEL_PATH_NOZOMI: &str = concatcp!("models/Nozomi.hierarchy", QUERY, VERSION);
#[rustfmt::skip] const TEXTURE_PATH_TRAIN_ICON: &str = concatcp!("textures/Train_Icon.sprite", QUERY, VERSION);

const NUM_SOUND_VO_TITLE: usize = 2;
const SOUND_PATH_VO_TITLES: [&str; NUM_SOUND_VO_TITLE] =
    [SOUND_PATH_HIKARI_TITLE, SOUND_PATH_NOZOMI_TITLE];

const NUM_SOUND_VO_START: usize = 4;
const SOUND_PATH_VO_STARTS: [&str; NUM_SOUND_VO_START] = [
    SOUND_PATH_VO_START_00,
    SOUND_PATH_VO_START_01,
    SOUND_PATH_VO_START_02,
    SOUND_PATH_VO_START_03,
];

const NUM_SOUND_VO_DAMAGED: usize = 6;
const SOUND_PATH_VO_DAMAGEDS: [&str; NUM_SOUND_VO_DAMAGED] = [
    SOUND_PATH_VO_DAMAGED_00,
    SOUND_PATH_VO_DAMAGED_01,
    SOUND_PATH_VO_DAMAGED_02,
    SOUND_PATH_VO_DAMAGED_03,
    SOUND_PATH_VO_DAMAGED_04,
    SOUND_PATH_VO_DAMAGED_05,
];
lazy_static! {
    static ref SOUND_DAMAGED_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_SOUND_VO_DAMAGED] = [5, 5, 5, 5, 1, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
}

const NUM_SOUND_VO_HEALINGS: usize = 4;
const SOUND_PATH_VO_HEALINGS: [&str; NUM_SOUND_VO_HEALINGS] = [
    SOUND_PATH_VO_HEALING_00,
    SOUND_PATH_VO_HEALING_01,
    SOUND_PATH_VO_HEALING_02,
    SOUND_PATH_VO_HEALING_03,
];

const NUM_SOUND_VO_INVINCIBLES: usize = 4;
const SOUND_PATH_VO_INVINCIBLES: [&str; NUM_SOUND_VO_INVINCIBLES] = [
    SOUND_PATH_VO_INVINCIBLE_00,
    SOUND_PATH_VO_INVINCIBLE_01,
    SOUND_PATH_VO_INVINCIBLE_02,
    SOUND_PATH_VO_INVINCIBLE_03,
];

const NUM_SOUND_VO_RESULTS: usize = 4;
const SOUND_PATH_VO_RESULTS: [&str; NUM_SOUND_VO_RESULTS] = [
    SOUND_PATH_VO_RESULT_00,
    SOUND_PATH_VO_RESULT_01,
    SOUND_PATH_VO_RESULT_02,
    SOUND_PATH_VO_RESULT_03,
];

const NUM_SOUND_VO_AOBA: usize = 2;
const SOUND_PATH_VO_AOBAS: [&str; NUM_SOUND_VO_AOBA] =
    [SOUND_PATH_VO_AOBA_00, SOUND_PATH_VO_AOBA_01];

const NUM_SOUND_VO_AOBA_HIT: usize = 2;
const SOUND_PATH_VO_AOBA_HITS: [&str; NUM_SOUND_VO_AOBA_HIT] =
    [SOUND_PATH_VO_AOBA_HIT_00, SOUND_PATH_VO_AOBA_HIT_01];

// --- CONSTANTS ---

#[cfg(target_arch = "wasm32")]
const HIGH_SCORE_KEY: &str = "high_score";

#[cfg(target_arch = "wasm32")]
const SYSTEM_VOLUME_KEY: &str = "system_volume";

const NUM_LANES: usize = 3;
const MAX_LANE_INDEX: usize = NUM_LANES - 1;
const LANE_LOCATIONS: [f32; NUM_LANES] = [-3.0, 0.25, 3.5];
const PLAYER_MIN_Z_POS: f32 = -20.0;
const PLAYER_MAX_Z_POS: f32 = -7.5;
const LANE_CHANGE_SPEED: f32 = 5.0;
const JUMP_STRENGTH: f32 = 12.5;
const GRAVITY: f32 = -30.0;
const FUEL_USAGE: f32 = 100.0 / 20.0;

const ATTACKED_DURATION: f32 = 3.0;
const INVINCIBLE_DURATION: f32 = 8.0;
const PREPARE_ANIM_DURATION: f32 = 1.0;
const FINISH_ANIM_DURATION: f32 = 1.0;

const DESPAWN_LOCATION: f32 = -100.0;
const SPAWN_LOCATION: f32 = 100.0;
const PLANE_SPAWN_INTERVAL: f32 = 30.0;

const NUM_OBJECTS: usize = 5;
const OBJECT_SPAWN_INTERVAL: f32 = 25.0;
const OBJECT_SPAWN_OFFSET: RangeInclusive<f32> = -5.0..=5.0;

const NUM_BARRICADE_LOCATIONS: usize = 7;
const NUM_STONE_LOCATIONS: usize = 7;
const NUM_FUEL_LOCATIONS: usize = 3;
const NUM_BELL_LOCATIONS: usize = 3;
const NUM_AOBA_LOCATIONS: usize = 3;

const BARRICADE_AMOUNT: f32 = 20.0;
const STONE_AMOUNT: f32 = 30.0;
const FUEL_AMOUNT: f32 = 30.0;

lazy_static! {
    static ref OBJECT_MODELS: HashMap<Object, &'static str> = {
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
    static ref OBJECT_COLLIDER: HashMap<Object, Collider> = {
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
                    offset: Vec3::new(0.0, 0.0, 0.0),
                    size: Vec3::new(0.5, 1.0, 0.5),
                },
            ),
        ]
        .into_iter()
        .collect();

        assert!(map.len() == NUM_OBJECTS);
        map
    };
    static ref SPAWN_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_OBJECTS] = [400, 300, 200, 95, 5];
        WeightedIndex::new(WEIGHTS).unwrap()
    };
}

const OBJECT_LIST: [Object; NUM_OBJECTS] = [
    Object::Barricade,
    Object::Stone,
    Object::Fuel,
    Object::Bell,
    Object::Aoba,
];
lazy_static! {
    static ref BARRICADE_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_BARRICADE_LOCATIONS] = [3, 3, 2, 3, 2, 2, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };

    static ref BARRICADE_LOCATIONS: [Vec<f32>; NUM_BARRICADE_LOCATIONS] = [
        vec![LANE_LOCATIONS[0]], // Pattern 1: Lane 0
        vec![LANE_LOCATIONS[1]], // Pattern 2: Lane 1
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1]], // Pattern 3: Lanes 0, 1
        vec![LANE_LOCATIONS[2]], // Pattern 4: Lane 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[2]], // Pattern 5: Lanes 0, 2
        vec![LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 6: Lanes 1, 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 7: Lanes 0, 1, 2
    ];

    static ref STONE_WEIGHTS: WeightedIndex<u32> = {
        const WEIGHTS: [u32; NUM_STONE_LOCATIONS] = [3, 3, 2, 3, 2, 2, 1];
        WeightedIndex::new(WEIGHTS).unwrap()
    };

    static ref STONE_LOCATIONS: [Vec<f32>; NUM_STONE_LOCATIONS] = [
        vec![LANE_LOCATIONS[0]], // Pattern 1: Lane 0
        vec![LANE_LOCATIONS[1]], // Pattern 2: Lane 1
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1]], // Pattern 3: Lanes 0, 1
        vec![LANE_LOCATIONS[2]], // Pattern 4: Lane 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[2]], // Pattern 5: Lanes 0, 2
        vec![LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 6: Lanes 1, 2
        vec![LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]], // Pattern 7: Lanes 0, 1, 2
    ];
}

const FUEL_LOCATIONS: [f32; NUM_FUEL_LOCATIONS] =
    [LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]];

const BELL_LOCATIONS: [f32; NUM_BELL_LOCATIONS] =
    [LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]];

const AOBA_LOCATIONS: [f32; NUM_AOBA_LOCATIONS] =
    [LANE_LOCATIONS[0], LANE_LOCATIONS[1], LANE_LOCATIONS[2]];

const MIN_SPEED: f32 = 20.0;
const MAX_SPEED: f32 = 30.0;
const INVINCIBLE_SPEED: f32 = 2.0 * MAX_SPEED;
const ACCELERATION: f32 = (MAX_SPEED - MIN_SPEED) / 30.0;

const SCORE_LIMITS: u32 = 999_999;
const FUEL_LIMITS: f32 = 100.0;
const INPUT_DELAY_TIME: f32 = 0.25;
const POINT_PER_DIST: f32 = 1.0;

const FUEL_DECO_CYCLE: f32 = PI * 1.0;
const ATTACKED_EFFECT_CYCLE: f32 = PI * 8.0;
const MIN_INVINCIBLE_EFFECT_CYCLE: f32 = PI * 4.0;
const MAX_INVINCIBLE_EFFECT_CYCLE: f32 = PI * 8.0;
const PAUSE_TITLE_CYCLE: f32 = 1.5;

const LANGUAGE_BTN_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const SLIDER_RAIL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const SLIDER_HANDLE_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const CLEAR_COLOR: Color = Color::srgb(0.48627, 0.81568, 1.0);
const LOADING_BAR_COLOR: Color = Color::srgb(0.2, 0.8, 0.2); // Green color for the loading bar.
const RESUME_BTN_COLOR: Color = Color::WHITE;
const OPTION_BTN_COLOR: Color = Color::WHITE;
const RESTART_BTN_COLOR: Color = Color::WHITE;
const EXIT_BTN_COLOR: Color = Color::srgb(0.98039, 0.37254, 0.33333);
const BACK_BTN_COLOR: Color = Color::srgb(0.98039, 0.37254, 0.33333);
const PAUSE_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
const PAUSE_BTN_COLOR: Color = Color::WHITE;
const PAUSE_ICON_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const FUEL_COLOR: Color = Color::srgb(0.18823, 0.21568, 0.27450);
const FUEL_GOOD_GAUGE_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
const FUEL_FAIR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.8, 0.2);
const FUEL_POOR_GAUGE_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);

// --- STATES ---

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    Error,
    Option,
    Pause,
    Resume,
    #[default]
    Setup,
    Initialize,
    LoadTitle,
    InitTitle,
    Title,
    Title2InGame,
    LoadInGame,
    InitInGame,
    ExitInGame,
    InitResult,
    PrepareInGame,
    StartInGame,
    InGame,
    WrapUpInGame,
    FinishedInGame,
    StartResult,
    Start2End,
    EndResult,
    CleanUpInGame,
    RestartResult,
    ExitResult,
}

// --- COMPONENTS ---

#[derive(Component)]
pub struct BackgroundSound;

#[derive(Component)]
pub struct EffectSound;

#[derive(Component)]
pub struct VoiceSound;

#[derive(Component)]
pub struct TrainSoundStart;

#[derive(Component)]
pub struct TrainSoundLoop1;

#[derive(Component)]
pub struct TrainSoundLoop2;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct ToyTrain0;

#[derive(Component)]
pub struct ToyTrain1;

#[derive(Component)]
pub struct ToyTrain2;

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

#[derive(Component)]
pub struct NewRecord;

#[derive(Component)]
pub struct SpawnRequest;

#[derive(Component)]
pub struct LoadingStateRoot;

#[derive(Component)]
pub struct OptionStateRoot;

#[derive(Component)]
pub struct TitleStateRoot;

#[derive(Component)]
pub struct InGameStateEntity;

#[derive(Component)]
pub struct InGameStateRoot;

#[derive(Component)]
pub struct ResultStateEntity;

#[derive(Component)]
pub struct ResultStateRoot;

#[derive(Component)]
pub struct Nozomi;

#[derive(Component)]
pub struct Hikari;

#[derive(Component)]
pub struct GlowRoot;

/// A marker component for the "Now Loading..." text UI entity.
#[derive(Component)]
pub struct LoadingText;

/// A marker component for the loading bar UI entity.
#[derive(Component)]
pub struct LoadingBar;

#[derive(Clone, Copy, Component, PartialEq, Eq, Hash)]
pub enum UI {
    SliderRail,
    OptionModal,
    BgmLabel,
    BgmVolume,
    BgmVolumeCursor,
    SfxLabel,
    SfxVolume,
    SfxVolumeCursor,
    VoiceLabel,
    VoiceVolume,
    VoiceVolumeCursor,
    LanguageEn,
    LanguageJa,
    LanguageKo,
    BackButton,

    HighScore,
    StartButton,
    OptionButton,
    StartLabel,
    FinishLabel,
    PauseButton,
    Score,
    Fuel,

    Pause,
    ResumeButton,
    InGameExitButton,

    ResumeCount1,
    ResumeCount2,
    ResumeCount3,

    ResultText,
    ResultImgFont,
    ResultModal,
    RestartButton,
    ResultExitButton,
    PlayTime,
    GameScore,
    BestScore,
    NewRecord,
}

#[derive(Component)]
pub struct Player;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Object {
    #[default]
    Barricade,
    Stone,
    Fuel,
    Bell,
    Aoba,
}

#[derive(Component)]
pub struct RotateAnimation {
    axis: Vec3,
    radian_per_sec: f32,
}

impl RotateAnimation {
    pub fn from_rotation_y(radian_per_sec: f32) -> Self {
        Self {
            axis: Vec3::Y,
            radian_per_sec,
        }
    }
}

#[derive(Component)]
pub struct FadeInAnimation {
    duration: f32,
    elapsed_time: f32,
}

impl FadeInAnimation {
    pub fn new(duration: f32) -> Self {
        #[cfg(not(feature = "no-debuging-assert"))]
        assert!(duration > 0.0);

        Self {
            duration,
            elapsed_time: 0.0,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
    }

    pub fn color(&self) -> Color {
        let t = (self.elapsed_time / self.duration).min(1.0);
        let alpha = (t - 1.0).powi(3) * (1.0 - t) + 1.0;
        Color::srgba(1.0, 1.0, 1.0, alpha)
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed_time >= self.duration
    }
}

#[derive(Component)]
pub struct FadeInOutAnimation {
    duration: f32,
    elapsed_time: f32,
}

impl FadeInOutAnimation {
    pub fn new(duration: f32) -> Self {
        #[cfg(not(feature = "no-debuging-assert"))]
        assert!(duration > 0.0);

        Self {
            duration,
            elapsed_time: 0.0,
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
    }

    pub fn color(&self) -> Color {
        let t = (self.elapsed_time / self.duration).min(1.0);
        let alpha = (t * PI).sin();
        Color::WHITE.with_alpha(alpha)
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed_time >= self.duration
    }
}

#[derive(Component, Clone, Copy)]
pub enum ResizableFont {
    Vertical { base: f32, size: f32 },
}

impl ResizableFont {
    pub fn vertical(base: f32, size: f32) -> Self {
        Self::Vertical { base, size }
    }
}

// --- RESOURCES ---

#[derive(Default, Resource)]
pub struct HighScore(pub u32);

#[derive(Default, Resource, Deref, DerefMut)]
pub struct Counter(pub u32);

#[derive(Resource)]
pub struct SceneTimer {
    elapsed_time: f32,
}

impl SceneTimer {
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
pub struct CurrentLane {
    index: usize,
}

impl Default for CurrentLane {
    fn default() -> Self {
        Self {
            index: NUM_LANES / 2,
        }
    }
}

#[derive(Resource)]
pub struct ForwardMovement {
    velocity: f32,
}

impl ForwardMovement {
    pub fn accel(&mut self, elapsed: f32) {
        let amount = ACCELERATION * elapsed;
        self.velocity = (self.velocity + amount).min(MAX_SPEED);
    }

    pub fn decel(&mut self, elapsed: f32) {
        let amount = ACCELERATION * elapsed;
        self.velocity = (self.velocity - amount).max(0.0);
    }

    pub fn reset(&mut self) {
        self.velocity = MIN_SPEED;
    }

    pub fn percentage(&self) -> f32 {
        ((self.velocity - MIN_SPEED) / (MAX_SPEED - MIN_SPEED)).clamp(0.0, 1.0)
    }
}

impl Default for ForwardMovement {
    fn default() -> Self {
        Self {
            velocity: MIN_SPEED,
        }
    }
}

#[derive(Resource)]
pub struct VerticalMovement {
    velocity: f32,
}

impl VerticalMovement {
    pub fn jump(&mut self) {
        self.velocity = JUMP_STRENGTH;
    }

    pub fn on_advanced(&mut self, elapsed: f32) {
        let amount = GRAVITY * elapsed;
        self.velocity += amount;
    }

    pub fn reset(&mut self) {
        self.velocity = 0.0;
    }
}

impl Default for VerticalMovement {
    fn default() -> Self {
        Self { velocity: 0.0 }
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

    pub fn on_advanced(&mut self, forward_move: &ForwardMovement, elapsed: f32) {
        self.distance += forward_move.velocity * elapsed;
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
    pub fn add(&mut self, amount: f32) {
        self.remaining = (self.remaining + amount).min(FUEL_LIMITS);
    }

    pub fn sub(&mut self, amount: f32) {
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
        current: &ForwardMovement,
        elapsed: f32,
    ) {
        let mut rng = rand::rng();
        self.distance += current.velocity * elapsed;
        while self.distance >= OBJECT_SPAWN_INTERVAL {
            let path = OBJECT_MODELS.get(&self.next_obj).cloned().unwrap();
            let collider = OBJECT_COLLIDER.get(&self.next_obj).cloned().unwrap();
            let model = asset_server.load(path);
            let delta = OBJECT_SPAWN_INTERVAL - self.distance;

            match self.next_obj {
                Object::Barricade => {
                    let index = BARRICADE_WEIGHTS.sample(&mut rng);
                    let locations = &BARRICADE_LOCATIONS[index];
                    for &lane_x in locations {
                        let recycle = self
                            .retired
                            .get_mut(&self.next_obj)
                            .and_then(|entities| entities.pop_front());

                        match recycle {
                            Some(entity) => {
                                info!("Recycle Barricade entity");
                                commands
                                    .entity(entity)
                                    .insert(Transform::from_xyz(
                                        lane_x,
                                        0.0,
                                        SPAWN_LOCATION + delta,
                                    ))
                                    .insert(self.next_obj);
                            }
                            None => {
                                commands.spawn((
                                    SpawnModel(model.clone()),
                                    Transform::from_xyz(lane_x, 0.0, SPAWN_LOCATION + delta),
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
                    let locations = &STONE_LOCATIONS[index];
                    for &lane_x in locations {
                        let recycle = self
                            .retired
                            .get_mut(&self.next_obj)
                            .and_then(|entities| entities.pop_front());

                        match recycle {
                            Some(entity) => {
                                info!("Recycle Stone entity");
                                commands
                                    .entity(entity)
                                    .insert(Transform::from_xyz(
                                        lane_x,
                                        0.0,
                                        SPAWN_LOCATION + delta,
                                    ))
                                    .insert(self.next_obj);
                            }
                            None => {
                                commands.spawn((
                                    SpawnModel(model.clone()),
                                    Transform::from_xyz(lane_x, 0.0, SPAWN_LOCATION + delta),
                                    InGameStateRoot,
                                    self.next_obj,
                                    collider,
                                ));
                            }
                        }
                    }
                }
                Object::Fuel => {
                    let lane_x = FUEL_LOCATIONS.choose(&mut rng).copied().unwrap();
                    let recycle = self
                        .retired
                        .get_mut(&self.next_obj)
                        .and_then(|entities| entities.pop_front());

                    match recycle {
                        Some(entity) => {
                            info!("Recycle Fuel entity");
                            commands
                                .entity(entity)
                                .insert(Transform::from_xyz(lane_x, 0.5, SPAWN_LOCATION + delta))
                                .insert(self.next_obj);
                        }
                        None => {
                            commands.spawn((
                                SpawnModel(model.clone()),
                                Transform::from_xyz(lane_x, 0.5, SPAWN_LOCATION + delta),
                                RotateAnimation::from_rotation_y(120f32.to_radians()),
                                InGameStateRoot,
                                self.next_obj,
                                collider,
                            ));
                        }
                    }
                }
                Object::Bell => {
                    let lane_x = BELL_LOCATIONS.choose(&mut rng).copied().unwrap();
                    let recycle = self
                        .retired
                        .get_mut(&self.next_obj)
                        .and_then(|entities| entities.pop_front());

                    match recycle {
                        Some(entity) => {
                            info!("Recycle Bell entity");
                            commands
                                .entity(entity)
                                .insert(Transform::from_xyz(lane_x, 0.5, SPAWN_LOCATION + delta))
                                .insert(self.next_obj);
                        }
                        None => {
                            commands.spawn((
                                SpawnModel(model.clone()),
                                Transform::from_xyz(lane_x, 0.5, SPAWN_LOCATION + delta),
                                RotateAnimation::from_rotation_y(120f32.to_radians()),
                                InGameStateRoot,
                                self.next_obj,
                                collider,
                            ));
                        }
                    }
                }
                Object::Aoba => {
                    let lane_x = AOBA_LOCATIONS.choose(&mut rng).copied().unwrap();
                    let recycle = self
                        .retired
                        .get_mut(&self.next_obj)
                        .and_then(|entities| entities.pop_front());

                    match recycle {
                        Some(entity) => {
                            info!("Recycle Aoba entity");
                            commands
                                .entity(entity)
                                .insert(Transform::from_xyz(lane_x, 0.0, SPAWN_LOCATION + delta))
                                .insert(AnimationClipHandle(asset_server.load(ANIM_PATH_AOBA)))
                                .insert(self.next_obj);
                        }
                        None => {
                            commands
                                .spawn((
                                    Transform::from_xyz(lane_x, 0.0, SPAWN_LOCATION + delta),
                                    InGameStateRoot,
                                    self.next_obj,
                                    collider,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        SpawnModel(model.clone()),
                                        AnimationClipHandle(asset_server.load(ANIM_PATH_AOBA)),
                                        Transform::IDENTITY
                                            .looking_to(*in_game::IN_GAME_AOBA_DIR, Vec3::Y),
                                        InGameStateEntity,
                                    ));

                                    let direction = in_game::IN_GAME_AOBA_DIR.clone();
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
        self.retired
            .entry(obj)
            .and_modify(|entities| {
                entities.push_back(entity);
            })
            .or_insert(VecDeque::from_iter([entity]));
        commands.entity(entity).remove::<Object>();
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

// --- UPDATE SYSTEMS ---

fn initialize_font_size(
    windows: Query<&Window>,
    mut query: Query<(&mut TextFont, &ResizableFont), Added<ResizableFont>>,
) {
    let window = windows.single().unwrap();
    for (mut font, &resizable) in query.iter_mut() {
        match resizable {
            ResizableFont::Vertical { base, size } => {
                let font_size = window.height() / base * size;
                font.font_size = font_size;
            }
        }
    }
}

fn update_font_size(
    mut reader: EventReader<WindowResized>,
    mut query: Query<(&mut TextFont, &ResizableFont)>,
) {
    for event in reader.read() {
        for (mut font, &resizable) in query.iter_mut() {
            match resizable {
                ResizableFont::Vertical { base, size } => {
                    let font_size = event.height / base * size;
                    font.font_size = font_size;
                }
            }
        }
    }
}

// --- UTILITY FUNCTIONS ---

#[cfg(target_arch = "wasm32")]
fn get_local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}
