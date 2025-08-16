use bevy::{
    asset::{Asset, AssetLoader, Handle, LoadContext, io::Reader},
    image::Image,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

/// A serializable representation of a material.
///
/// This struct is used to load material data from a file (e.g., JSON).
/// It can then be converted into a `MaterialAsset` for use in the Bevy engine.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableMaterial {
    /// The path to the base color texture for this material.
    pub base_color_texture: Option<String>,
    /// The metallic value for this material.
    pub metallic: Option<f32>,
    /// The roughness value for this material.
    pub roughness: Option<f32>,
}

/// A material asset.
///
/// This struct holds the data for a material that can be used in the Bevy engine.
#[derive(Asset, TypePath)]
pub struct MaterialAsset {
    /// The base color texture for this material.
    pub base_color_texture: Option<Handle<Image>>,
    /// The metallic value for this material.
    pub metallic: f32,
    /// The roughness value for this material.
    pub roughness: f32,
}

/// An error that can occur when loading a material.
#[derive(Debug, thiserror::Error)]
pub enum MaterialLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for material assets.
///
/// This struct implements the `AssetLoader` trait for `MaterialAsset`.
#[derive(Default)]
pub struct MaterialAssetLoader;

impl AssetLoader for MaterialAssetLoader {
    type Asset = MaterialAsset;
    type Settings = ();
    type Error = MaterialLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Deserialize the bytes into a `SerializableMaterial`.
            let serializable: SerializableMaterial = serde_json::from_slice(&bytes)?;

            // Load the base color texture, if it exists.
            let base_color_texture = serializable
                .base_color_texture
                .as_ref()
                .map(|path| load_context.load(path));

            // Create the `MaterialAsset`.
            Ok(MaterialAsset {
                base_color_texture,
                metallic: serializable.metallic.unwrap_or(0.0),
                roughness: serializable.roughness.unwrap_or(0.5),
            })
        })
    }
}
