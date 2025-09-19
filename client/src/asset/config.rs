use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

#[derive(Asset, TypePath, Deserialize)]
pub struct Configuration {
    pub server_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct ConfigAssetLoader;

impl AssetLoader for ConfigAssetLoader {
    type Asset = Configuration;
    type Settings = ();
    type Error = ConfigurationLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let data: Configuration = serde_json::from_slice(&bytes)?;
            Ok(data)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
