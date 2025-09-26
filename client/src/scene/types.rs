use std::f32::consts::PI;

use bevy::prelude::*;

use super::*;

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
pub struct Lane {
    index: usize,
}

impl Lane {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn get(&self) -> usize {
        self.index
    }

    pub fn inc(&mut self) {
        self.index = (self.index + 1).min(MAX_LANE_INDEX);
    }

    pub fn dec(&mut self) {
        self.index = self.index.saturating_sub(1);
    }
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            index: MAX_LANE_INDEX / 2,
        }
    }
}

#[derive(Component)]
pub struct ForwardMovement(f32);

impl ForwardMovement {
    pub fn new(velocity: f32) -> Self {
        Self(velocity)
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, velocity: f32) {
        self.0 = velocity;
    }
}

#[derive(Component)]
pub struct VerticalMovement(f32);

impl VerticalMovement {
    pub fn new(velocity: f32) -> Self {
        Self(velocity)
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, velocity: f32) {
        self.0 = velocity;
    }
}

#[derive(Component)]

pub struct Acceleration(f32);

impl Acceleration {
    pub fn new(amount: f32) -> Self {
        Self(amount)
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

#[derive(Component)]
pub struct BaseColor(pub Color);

#[derive(Component)]
pub struct RotateAnimation {
    pub axis: Vec3,
    pub radian_per_sec: f32,
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
