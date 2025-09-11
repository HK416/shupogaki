use bevy::prelude::*;

#[derive(Resource)]
pub struct SystemVolume {
    pub background: u8,
    pub effect: u8,
    pub voice: u8,
}

impl SystemVolume {
    pub fn background_percentage(&self) -> f32 {
        self.background as f32 / 255.0
    }

    pub fn effect_percentage(&self) -> f32 {
        self.effect as f32 / 255.0
    }

    pub fn voice_percentage(&self) -> f32 {
        self.voice as f32 / 255.0
    }
}

impl Default for SystemVolume {
    fn default() -> Self {
        Self {
            background: 204,
            effect: 204,
            voice: 204,
        }
    }
}
