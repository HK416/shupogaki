use bevy::{
    animation::{AnimationTargetId, animated_field},
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::{AsyncComputeTaskPool, ConditionalSendFuture},
};
use serde::Deserialize;

use crate::asset::{Float3, Float4};

use super::*;

/// A component that holds a handle to an animation clip.
/// This is used to trigger the animation playback once the model is loaded.
#[derive(Component)]
pub struct AnimationClipHandle(pub Handle<AnimationClip>);

/// A serializable representation of an animation clip, designed for loading from a file.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableAnimation {
    /// The total duration of the animation in seconds.
    pub duration: f32,
    /// The collection of animation curves that make up this clip.
    pub curves: Vec<SerializableAnimCurve>,
}

/// A serializable animation curve that defines the animation for a single bone/target.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableAnimCurve {
    /// The name of the bone or target that this curve animates.
    pub bone: String,
    /// The timestamps for each keyframe, in seconds.
    pub timestamps: Vec<f32>,
    /// The keyframe data corresponding to the timestamps.
    pub keyframes: Vec<SerializableKeyframe>,
}

/// A serializable keyframe, representing the state of a target at a specific time.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableKeyframe {
    /// The translation (position) of the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<Float3>,
    /// The rotation (as a quaternion) of the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Float4>,
    /// The scale of the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Float3>,
}

/// An error that can occur when loading a `.anim` asset.
#[derive(Debug, thiserror::Error)]
pub enum AnimationLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to decrypt asset for the following reason:{0}")]
    Crypt(#[from] anyhow::Error),
}

/// A loader for `.anim` assets.
#[derive(Default)]
pub struct AnimationAssetLoader;

impl AssetLoader for AnimationAssetLoader {
    type Asset = AnimationClip;
    type Settings = ();
    type Error = AnimationLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        info!("asset load: {}", load_context.asset_path());
        Box::pin(async move {
            // Read the raw bytes from the asset file.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let pool = AsyncComputeTaskPool::get();
            let decrypted_data = pool
                .spawn(async move {
                    let key = reconstruct_key();
                    decrypt_bytes(&bytes, &key)
                })
                .await?;

            // Deserialize the bytes from JSON into a `SerializableAnimation`.
            let serializable: SerializableAnimation = serde_json::from_slice(&decrypted_data)?;

            // Convert the serializable format into a Bevy `AnimationClip`.
            Ok(create_animation_clip(&serializable))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

/// Creates a Bevy `AnimationClip` from a `SerializableAnimation`.
///
/// This function iterates through the serializable curves and converts them into
/// Bevy's native animation curve format, adding them to the clip.
pub fn create_animation_clip(serializable: &SerializableAnimation) -> AnimationClip {
    let mut clip = AnimationClip::default();

    // Set the total duration of the clip.
    let duration_seconds = serializable.duration;
    clip.set_duration(duration_seconds);

    // Process each curve (for each bone) in the serializable animation.
    for curve in &serializable.curves {
        let bone_name = &curve.bone;
        process_translation_curve(&mut clip, bone_name, curve);
        process_rotation_curve(&mut clip, bone_name, curve);
        process_scale_curve(&mut clip, bone_name, curve);
    }

    clip
}

/// Processes and adds the translation keyframes from a `SerializableAnimCurve` to an `AnimationClip`.
fn process_translation_curve(
    clip: &mut AnimationClip,
    bone_name: &str,
    curve: &SerializableAnimCurve,
) {
    let mut translation_keyframes: Vec<(f32, Vec3)> = Vec::with_capacity(curve.keyframes.len());

    // Collect all translation keyframes from the serializable data.
    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i)
            && let Some(translation) = keyframe.translation
        {
            translation_keyframes.push((timestamp, translation.into()));
        }
    }

    // If any translation keyframes were found, create a curve and add it to the clip.
    if !translation_keyframes.is_empty() {
        clip.add_curve_to_target(
            AnimationTargetId::from_name(&Name::new(bone_name.to_string())),
            AnimatableCurve::new(
                animated_field!(Transform::translation),
                UnevenSampleAutoCurve::new(translation_keyframes).expect(
                    "should be able to build translation curve because we pass in valid samples",
                ),
            ),
        );
    }
}

/// Processes and adds the rotation keyframes from a `SerializableAnimCurve` to an `AnimationClip`.
fn process_rotation_curve(
    clip: &mut AnimationClip,
    bone_name: &str,
    curve: &SerializableAnimCurve,
) {
    let mut rotation_keyframes: Vec<(f32, Quat)> = Vec::with_capacity(curve.keyframes.len());

    // Collect all rotation keyframes from the serializable data.
    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i)
            && let Some(rotation) = keyframe.rotation
        {
            rotation_keyframes.push((timestamp, rotation.into()));
        }
    }

    // If any rotation keyframes were found, create a curve and add it to the clip.
    if !rotation_keyframes.is_empty() {
        clip.add_curve_to_target(
            AnimationTargetId::from_name(&Name::new(bone_name.to_string())),
            AnimatableCurve::new(
                animated_field!(Transform::rotation),
                UnevenSampleAutoCurve::new(rotation_keyframes).expect(
                    "should be able to build rotation curve because we pass in valid samples",
                ),
            ),
        );
    }
}

/// Processes and adds the scale keyframes from a `SerializableAnimCurve` to an `AnimationClip`.
fn process_scale_curve(clip: &mut AnimationClip, bone_name: &str, curve: &SerializableAnimCurve) {
    let mut scale_keyframes: Vec<(f32, Vec3)> = Vec::with_capacity(curve.keyframes.len());

    // Collect all scale keyframes from the serializable data.
    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i)
            && let Some(scale) = keyframe.scale
        {
            scale_keyframes.push((timestamp, scale.into()));
        }
    }

    // If any scale keyframes were found, create a curve and add it to the clip.
    if !scale_keyframes.is_empty() {
        clip.add_curve_to_target(
            AnimationTargetId::from_name(&Name::new(bone_name.to_string())),
            AnimatableCurve::new(
                animated_field!(Transform::scale),
                UnevenSampleAutoCurve::new(scale_keyframes)
                    .expect("should be able to build scale curve because we pass in valid samples"),
            ),
        );
    }
}
