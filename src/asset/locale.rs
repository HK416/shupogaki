use std::{collections::HashMap, fmt};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

#[derive(Default, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    #[default]
    En,
    Ja,
    Ko,
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Locale::En => write!(f, "en"),
            Locale::Ja => write!(f, "jp"),
            Locale::Ko => write!(f, "ko"),
        }
    }
}

#[derive(Default, Resource)]
pub struct CurrentLocale(pub Locale);

#[derive(Default, Resource)]
pub struct LocalizationAssets {
    pub locale: HashMap<Locale, Handle<LocalizationData>>,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct LocalizationData(pub HashMap<String, String>);

#[derive(Debug, thiserror::Error)]
pub enum LocalizationDataLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct LocalizationDataLoader;

impl AssetLoader for LocalizationDataLoader {
    type Asset = LocalizationData;
    type Settings = ();
    type Error = LocalizationDataLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let data: LocalizationData = serde_json::from_slice(&bytes)?;
            Ok(data)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
