use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    pbr::ExtendedMaterial,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::{
    asset::Float4,
    shader::face_mouth::{FacialExpressionExtension, FacialExpressionUniform},
};

use super::*;

/// Represents the blend mode for a material, mapping to Bevy's `AlphaMode`.
#[derive(Debug, Deserialize, Clone)]
pub enum BlendMode {
    Opaque,
    Mask(f32),
    Blend,
    Premultiplied,
    AlphaToCoverage,
    Add,
    Multiply,
}

impl From<BlendMode> for AlphaMode {
    fn from(value: BlendMode) -> Self {
        match value {
            BlendMode::Opaque => AlphaMode::Opaque,
            BlendMode::Mask(mask) => AlphaMode::Mask(mask),
            BlendMode::Blend => AlphaMode::Blend,
            BlendMode::Premultiplied => AlphaMode::Premultiplied,
            BlendMode::AlphaToCoverage => AlphaMode::AlphaToCoverage,
            BlendMode::Add => AlphaMode::Add,
            BlendMode::Multiply => AlphaMode::Multiply,
        }
    }
}

/// A serializable representation of a material, designed for loading from a file.
///
/// This struct holds optional values for material properties, which are then
/// used to construct a Bevy `StandardMaterial`.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableMaterial {
    /// The base color of the material.
    #[serde(skip_serializing_if = "Option::is_none")]
    base_color: Option<Float4>,
    /// The name of the base color texture file (without path or extension).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<String>,
    /// The metallic value (0.0 to 1.0) for this material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f32>,
    /// The perceptual roughness value (0.0 to 1.0) for this material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflectance: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_color: Option<Float4>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_color_texture: Option<String>,

    /// Whether this material should be rendered as unlit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_sided: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mouth_altas: Option<String>,
}

/// An error that can occur when loading a `.material` asset.
#[derive(Debug, thiserror::Error)]
pub enum MaterialLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to decrypt asset for the following reason:{0}")]
    Crypt(#[from] anyhow::Error),
}

/// A loader for `.material` assets.
///
/// This struct implements the `AssetLoader` trait to load `.material` files,
/// deserialize them, and convert them into Bevy's `StandardMaterial`.
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
        info!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the raw bytes from the asset file.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let key = reconstruct_key();
            let decrypted_data = decrypt_bytes(&bytes, &key)?;

            // Deserialize the bytes from JSON into a `SerializableMaterial`.
            let serializable: SerializableMaterial = serde_json::from_slice(&decrypted_data)?;

            let base_color = serializable
                .base_color
                .as_ref()
                .map(|v| Color::srgba(v.x, v.y, v.z, v.w))
                .unwrap_or(Color::WHITE);

            // Load the base color texture as a dependency.
            // The material file specifies texture names without extensions or full paths.
            // We construct the full asset path here to load our custom `.texture` format.
            let base_color_texture = serializable
                .base_color_texture
                .as_ref()
                .map(|name| load_context.load(format!("textures/{}.texture", name)));

            let emissive_color_texture = serializable
                .emissive_color_texture
                .as_ref()
                .map(|name| load_context.load(format!("textures/{}.texture", name)));

            // Create the final `StandardMaterial` asset using the loaded data.
            // Default values are used if properties are not specified in the file.
            Ok(StandardMaterial {
                base_color,
                base_color_texture,
                metallic: serializable
                    .metallic
                    .map(|v| v.clamp(0.0, 1.0))
                    .unwrap_or(0.0),
                perceptual_roughness: serializable
                    .roughness
                    .map(|v| v.clamp(0.089, 1.0))
                    .unwrap_or(0.5),
                reflectance: serializable.reflectance.unwrap_or(0.5),
                emissive: serializable
                    .emissive_color
                    .map(|v| LinearRgba::new(v.x, v.y, v.z, v.w))
                    .unwrap_or(LinearRgba::BLACK),
                emissive_texture: emissive_color_texture,
                unlit: serializable.unlit.unwrap_or(false),
                double_sided: serializable.double_sided.unwrap_or(false),
                alpha_mode: serializable
                    .blend_mode
                    .map(|m| m.into())
                    .unwrap_or(AlphaMode::Opaque),
                ..Default::default()
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["material"]
    }
}

#[derive(Default)]
pub struct FaceMouthMaterialAssetLoader;

impl AssetLoader for FaceMouthMaterialAssetLoader {
    type Asset = ExtendedMaterial<StandardMaterial, FacialExpressionExtension>;
    type Settings = ();
    type Error = MaterialLoaderError;

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

            // Deserialize the bytes from JSON into a `SerializableMaterial`.
            let serializable: SerializableMaterial = serde_json::from_slice(&decrypted_data)?;

            let base_color = serializable
                .base_color
                .as_ref()
                .map(|v| Color::srgba(v.x, v.y, v.z, v.w))
                .unwrap_or(Color::WHITE);

            // Load the base color texture as a dependency.
            // The material file specifies texture names without extensions or full paths.
            // We construct the full asset path here to load our custom `.texture` format.
            let base_color_texture = serializable
                .base_color_texture
                .as_ref()
                .map(|name| load_context.load(format!("textures/{}.texture", name)));

            let emissive_color_texture = serializable
                .emissive_color_texture
                .as_ref()
                .map(|name| load_context.load(format!("textures/{}.texture", name)));

            let mouth_atlas = serializable
                .mouth_altas
                .as_ref()
                .map(|name| load_context.load(format!("textures/{}.texture", name)))
                .unwrap();

            // Create the final `StandardMaterial` asset using the loaded data.
            // Default values are used if properties are not specified in the file.
            Ok(ExtendedMaterial {
                base: StandardMaterial {
                    base_color,
                    base_color_texture,
                    metallic: serializable
                        .metallic
                        .map(|v| v.clamp(0.0, 1.0))
                        .unwrap_or(0.0),
                    perceptual_roughness: serializable
                        .roughness
                        .map(|v| v.clamp(0.089, 1.0))
                        .unwrap_or(0.5),
                    reflectance: serializable.reflectance.unwrap_or(0.5),
                    emissive: serializable
                        .emissive_color
                        .map(|v| LinearRgba::new(v.x, v.y, v.z, v.w))
                        .unwrap_or(LinearRgba::BLACK),
                    emissive_texture: emissive_color_texture,
                    unlit: serializable.unlit.unwrap_or(false),
                    double_sided: serializable.double_sided.unwrap_or(false),
                    alpha_mode: serializable
                        .blend_mode
                        .map(|m| m.into())
                        .unwrap_or(AlphaMode::Opaque),
                    ..Default::default()
                },
                extension: FacialExpressionExtension {
                    mouth_atlas,
                    uniform: FacialExpressionUniform::default(),
                },
            })
        })
    }
}
