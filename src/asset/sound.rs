use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};

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

#[derive(Debug, thiserror::Error)]
pub enum SoundLoaderError {
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
}

#[derive(Default)]
pub struct SoundAssetLoader;

impl AssetLoader for SoundAssetLoader {
    type Asset = AudioSource;
    type Settings = ();
    type Error = SoundLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제

            Ok(AudioSource {
                bytes: bytes.into(),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["sound"]
    }
}
