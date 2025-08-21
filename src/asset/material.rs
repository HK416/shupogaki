use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<String>,
    /// The metallic value for this material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f32>,
    /// The roughness value for this material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,
    /// Whether this material is unlit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlit: Option<bool>,
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
    type Asset = StandardMaterial;
    type Settings = ();
    type Error = MaterialLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        debug!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Deserialize the bytes into a `SerializableMaterial`.
            let serializable: SerializableMaterial = serde_json::from_slice(&bytes)?;

            // Create a `StandardMaterial` from the `SerializableMaterial`.
            // The material file specifies texture paths without extensions.
            // We append the `.texture` extension here to load our custom texture format.
            let base_color_texture = serializable
                .base_color_texture
                .as_ref()
                .map(|name| load_context.load(&format!("textures/{}.texture", name)));

            // Create the `MaterialAsset`.
            Ok(StandardMaterial {
                base_color_texture,
                metallic: serializable.metallic.unwrap_or(0.0),
                perceptual_roughness: serializable.roughness.unwrap_or(0.5),
                unlit: serializable.unlit.unwrap_or(false),
                ..Default::default()
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["material"]
    }
}
