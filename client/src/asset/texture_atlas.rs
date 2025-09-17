use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Atlas, UInt2};

use super::*;

/// Represents the serializable structure of a `.atlas` file.
/// This is used to deserialize the JSON data into a format that can be
/// used to construct a `TextureAtlasLayout`.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableTextureAtlas {
    /// The total size (width and height) of the texture atlas image.
    pub size: UInt2,
    /// A list of rectangles defining the position and size of each individual texture within the atlas.
    pub textures: Vec<Atlas>,
}

#[derive(Debug, thiserror::Error)]
pub enum TextureAtlasLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to decrypt asset for the following reason:{0}")]
    Crypt(#[from] anyhow::Error),
}

#[derive(Default)]
pub struct TextureAtlasAssetLoader;

impl AssetLoader for TextureAtlasAssetLoader {
    type Asset = TextureAtlasLayout;
    type Settings = ();
    type Error = TextureAtlasLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        info!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the raw bytes from the asset file.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let key = reconstruct_key();
            let decrypted_data = decrypt_bytes(&bytes, &key)?;

            // Deserialize the JSON bytes into our serializable format.
            let serializable: SerializableTextureAtlas = serde_json::from_slice(&decrypted_data)?;

            // Create a new, empty `TextureAtlasLayout` with the specified dimensions.
            let mut atlas_layout = TextureAtlasLayout::new_empty(serializable.size.into());
            // Add each texture rectangle from the deserialized data to the layout.
            for rect in serializable.textures.iter().cloned() {
                atlas_layout.add_texture(rect.into());
            }

            // Return the fully constructed `TextureAtlasLayout` asset.
            Ok(atlas_layout)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas"]
    }
}
