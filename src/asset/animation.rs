use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Float3, Float4};

/// A serializable animation that can be loaded from a file.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableAnimation {
    /// The name of the animation.
    pub name: String,
    /// The duration of the animation in seconds.
    pub duration: f32,
    /// The animation curves for each bone.
    pub curves: Vec<SerializableAnimCurve>,
}

/// A serializable animation curve that defines the animation of a single bone.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableAnimCurve {
    /// The name of the bone that this curve animates.
    pub bone: String,
    /// The timestamps of the keyframes.
    pub timestamps: Vec<f32>,
    /// The keyframes of the animation.
    pub keyframes: Vec<SerializableKeyframe>,
}

/// A serializable keyframe that defines the transformation of a bone at a specific time.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableKeyframe {
    /// The translation of the bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<Float3>,
    /// The rotation of the bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Float4>,
    /// The scale of the bone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Float3>,
}

/// An asset that represents a loaded animation.
#[derive(Asset, TypePath)]
pub struct AnimationAsset {
    /// The serializable animation data.
    pub serializable: SerializableAnimation,
}

/// An error that can occur when loading a animation.
#[derive(Debug, thiserror::Error)]
pub enum AnimationLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for animation assets.
#[derive(Default)]
pub struct AnimationAssetLoader;

impl AssetLoader for AnimationAssetLoader {
    type Asset = AnimationAsset;
    type Settings = ();
    type Error = AnimationLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Deserialize the bytes into a `SerializableAnimation`
            let serializable: SerializableAnimation = serde_json::from_slice(&bytes)?;

            // Create the `AnimationAsset`
            Ok(AnimationAsset { serializable })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}
