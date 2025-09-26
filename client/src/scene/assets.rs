use const_format::concatcp;

#[rustfmt::skip] pub const QUERY: &str = "?";
#[rustfmt::skip] pub const VERSION: &str = concat!("v=", env!("CARGO_PKG_VERSION_MINOR"));

#[rustfmt::skip] pub const LOCALE_PATH_EN: &str = concatcp!("locale/en.json", QUERY, VERSION);
#[rustfmt::skip] pub const LOCALE_PATH_JA: &str = concatcp!("locale/ja.json", QUERY, VERSION);
#[rustfmt::skip] pub const LOCALE_PATH_KO: &str = concatcp!("locale/ko.json", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NOTOSANS_BOLD: &str = concatcp!("fonts/NotoSans-Bold.otf", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_START: &str = concatcp!("fonts/ImgFont_Start.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_FINISH: &str = concatcp!("fonts/ImgFont_Finish.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_PAUSE: &str = concatcp!("fonts/ImgFont_Pause.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NEW: &str = concatcp!("fonts/ImgFont_New.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NUM_1: &str = concatcp!("fonts/ImgFont_1.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NUM_2: &str = concatcp!("fonts/ImgFont_2.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NUM_3: &str = concatcp!("fonts/ImgFont_3.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_TIME: &str = concatcp!("fonts/ImgFont_Time.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_SCORE: &str = concatcp!("fonts/ImgFont_Score.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_BEST: &str = concatcp!("fonts/ImgFont_Best.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const FONT_PATH_NUMBER: &str = concatcp!("fonts/ImgFont_Number.sprite", QUERY, VERSION);
#[rustfmt::skip] pub const ATLAS_PATH_NUMBER: &str = concatcp!("fonts/ImgFont_Number.atlas", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_HIKARI_TITLE: &str = concatcp!("sounds/Hikari_Title.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_NOZOMI_TITLE: &str = concatcp!("sounds/Nozomi_Title.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_BACKGROUND: &str = concatcp!("sounds/Theme_253_Game.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_UI_START: &str = concatcp!("sounds/UI_Start.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_UI_FINISH: &str = concatcp!("sounds/UI_Finish.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_UI_BUTTON_BACK: &str = concatcp!("sounds/UI_Button_Back.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_UI_BUTTON_TOUCH: &str = concatcp!("sounds/UI_Button_Touch.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_UI_LOADING: &str = concatcp!("sounds/UI_Loading.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_DOOR_BELL: &str = concatcp!("sounds/SFX_DoorBell.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_START: &str = concatcp!("sounds/SFX_Train_Start.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_LOOP_1: &str = concatcp!("sounds/SFX_Train_Loop_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_LOOP_2: &str = concatcp!("sounds/SFX_Train_Loop_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_END: &str = concatcp!("sounds/SFX_Train_End.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_LANDING: &str = concatcp!("sounds/SFX_Train_Landing.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_SFX_TRAIN_INVINCIBLE: &str = concatcp!("sounds/SFX_Train_Invincible.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_START_00: &str = concatcp!("sounds/VO_Start_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_START_01: &str = concatcp!("sounds/VO_Start_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_START_02: &str = concatcp!("sounds/VO_Start_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_START_03: &str = concatcp!("sounds/VO_Start_03.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_00: &str = concatcp!("sounds/VO_Damaged_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_01: &str = concatcp!("sounds/VO_Damaged_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_02: &str = concatcp!("sounds/VO_Damaged_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_03: &str = concatcp!("sounds/VO_Damaged_03.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_04: &str = concatcp!("sounds/VO_Damaged_04.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_DAMAGED_05: &str = concatcp!("sounds/VO_Damaged_05.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_HEALING_00: &str = concatcp!("sounds/VO_Healing_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_HEALING_01: &str = concatcp!("sounds/VO_Healing_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_HEALING_02: &str = concatcp!("sounds/VO_Healing_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_HEALING_03: &str = concatcp!("sounds/VO_Healing_03.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_INVINCIBLE_00: &str = concatcp!("sounds/VO_Invincible_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_INVINCIBLE_01: &str = concatcp!("sounds/VO_Invincible_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_INVINCIBLE_02: &str = concatcp!("sounds/VO_Invincible_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_INVINCIBLE_03: &str = concatcp!("sounds/VO_Invincible_03.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_RESULT_00: &str = concatcp!("sounds/VO_Result_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_RESULT_01: &str = concatcp!("sounds/VO_Result_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_RESULT_02: &str = concatcp!("sounds/VO_Result_02.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_RESULT_03: &str = concatcp!("sounds/VO_Result_03.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_AOBA_00: &str = concatcp!("sounds/VO_Aoba_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_AOBA_01: &str = concatcp!("sounds/VO_Aoba_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_AOBA_HIT_00: &str = concatcp!("sounds/VO_Aoba_Hit_00.sound", QUERY, VERSION);
#[rustfmt::skip] pub const SOUND_PATH_VO_AOBA_HIT_01: &str = concatcp!("sounds/VO_Aoba_Hit_01.sound", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_AOBA: &str = concatcp!("animations/Aoba.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_HIKARI_CAFE_IDLE: &str = concatcp!("animations/Hikari_Cafe_Idle.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_HIKARI_IN_GAME: &str = concatcp!("animations/Hikari_InGame.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_HIKARI_VICTORY_START: &str = concatcp!("animations/Hikari_Victory_Start_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_HIKARI_VICTORY_END: &str = concatcp!("animations/Hikari_Victory_End_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_NOZOMI_CAFE_IDLE: &str = concatcp!("animations/Nozomi_Cafe_Idle.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_NOZOMI_IN_GAME: &str = concatcp!("animations/Nozomi_InGame.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_NOZOMI_VICTORY_START: &str = concatcp!("animations/Nozomi_Victory_Start_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] pub const ANIM_PATH_NOZOMI_VICTORY_END: &str = concatcp!("animations/Nozomi_Victory_End_Interaction.anim", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_PLANE_0: &str = concatcp!("models/Plane_0.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_PLANE_999: &str = concatcp!("models/Plane_999.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_TOY_TRAIN_00: &str = concatcp!("models/ToyTrain00.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_TOY_TRAIN_01: &str = concatcp!("models/ToyTrain01.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_TOY_TRAIN_02: &str = concatcp!("models/ToyTrain02.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_BARRICADE: &str = concatcp!("models/Barricade.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_STONE: &str = concatcp!("models/Stone.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_FUEL: &str = concatcp!("models/Fuel.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_DOOR_BELL: &str = concatcp!("models/DoorBell.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_AOBA: &str = concatcp!("models/Aoba.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_GLOW: &str = concatcp!("models/Glow.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_HIKARI: &str = concatcp!("models/Hikari.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const MODEL_PATH_NOZOMI: &str = concatcp!("models/Nozomi.hierarchy", QUERY, VERSION);
#[rustfmt::skip] pub const TEXTURE_PATH_TRAIN_ICON: &str = concatcp!("textures/Train_Icon.sprite", QUERY, VERSION);

pub const NUM_SOUND_VO_TITLE: usize = 2;
pub const SOUND_PATH_VO_TITLES: [&str; NUM_SOUND_VO_TITLE] =
    [SOUND_PATH_HIKARI_TITLE, SOUND_PATH_NOZOMI_TITLE];

pub const NUM_SOUND_VO_START: usize = 4;
pub const SOUND_PATH_VO_STARTS: [&str; NUM_SOUND_VO_START] = [
    SOUND_PATH_VO_START_00,
    SOUND_PATH_VO_START_01,
    SOUND_PATH_VO_START_02,
    SOUND_PATH_VO_START_03,
];

pub const NUM_SOUND_VO_DAMAGED: usize = 6;
pub const SOUND_PATH_VO_DAMAGEDS: [&str; NUM_SOUND_VO_DAMAGED] = [
    SOUND_PATH_VO_DAMAGED_00,
    SOUND_PATH_VO_DAMAGED_01,
    SOUND_PATH_VO_DAMAGED_02,
    SOUND_PATH_VO_DAMAGED_03,
    SOUND_PATH_VO_DAMAGED_04,
    SOUND_PATH_VO_DAMAGED_05,
];

pub const NUM_SOUND_VO_HEALINGS: usize = 4;
pub const SOUND_PATH_VO_HEALINGS: [&str; NUM_SOUND_VO_HEALINGS] = [
    SOUND_PATH_VO_HEALING_00,
    SOUND_PATH_VO_HEALING_01,
    SOUND_PATH_VO_HEALING_02,
    SOUND_PATH_VO_HEALING_03,
];

pub const NUM_SOUND_VO_INVINCIBLES: usize = 4;
pub const SOUND_PATH_VO_INVINCIBLES: [&str; NUM_SOUND_VO_INVINCIBLES] = [
    SOUND_PATH_VO_INVINCIBLE_00,
    SOUND_PATH_VO_INVINCIBLE_01,
    SOUND_PATH_VO_INVINCIBLE_02,
    SOUND_PATH_VO_INVINCIBLE_03,
];

pub const NUM_SOUND_VO_RESULTS: usize = 4;
pub const SOUND_PATH_VO_RESULTS: [&str; NUM_SOUND_VO_RESULTS] = [
    SOUND_PATH_VO_RESULT_00,
    SOUND_PATH_VO_RESULT_01,
    SOUND_PATH_VO_RESULT_02,
    SOUND_PATH_VO_RESULT_03,
];

pub const NUM_SOUND_VO_AOBA: usize = 2;
pub const SOUND_PATH_VO_AOBAS: [&str; NUM_SOUND_VO_AOBA] =
    [SOUND_PATH_VO_AOBA_00, SOUND_PATH_VO_AOBA_01];

pub const NUM_SOUND_VO_AOBA_HIT: usize = 2;
pub const SOUND_PATH_VO_AOBA_HITS: [&str; NUM_SOUND_VO_AOBA_HIT] =
    [SOUND_PATH_VO_AOBA_HIT_00, SOUND_PATH_VO_AOBA_HIT_01];
