use bevy::{
    animation::{AnimationTargetId, animated_field},
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Float3, Float4};

/// A serializable animation that can be loaded from a file.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableAnimation {
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
    type Asset = AnimationClip;
    type Settings = ();
    type Error = AnimationLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        debug!("asset load: {}", load_context.asset_path());
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Deserialize the bytes into a `SerializableAnimation`
            let serializable: SerializableAnimation = serde_json::from_slice(&bytes)?;

            // Create the `AnimationAsset`
            Ok(create_animation_clip(&serializable))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

/// Creates a Bevy `AnimationClip` from a `SerializableAnimation`.
///
/// This function iterates through the animation curves of the asset and processes them
/// into translation, rotation, and scale curves for the `AnimationClip`.
pub fn create_animation_clip(serializable: &SerializableAnimation) -> AnimationClip {
    let mut clip = AnimationClip::default();

    let duration_seconds = serializable.duration;
    clip.set_duration(duration_seconds);

    for curve in &serializable.curves {
        let bone_name = &curve.bone;
        process_translation_curve(&mut clip, bone_name, &curve);
        process_rotation_curve(&mut clip, bone_name, &curve);
        process_scale_curve(&mut clip, bone_name, &curve);
    }

    clip
}

fn process_translation_curve(
    clip: &mut AnimationClip,
    bone_name: &str,
    curve: &SerializableAnimCurve,
) {
    let mut translation_keyframes: Vec<(f32, Vec3)> = Vec::with_capacity(curve.keyframes.len());

    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i) {
            if let Some(translation) = keyframe.translation {
                translation_keyframes.push((timestamp, translation.into()));
            }
        }
    }

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

fn process_rotation_curve(
    clip: &mut AnimationClip,
    bone_name: &str,
    curve: &SerializableAnimCurve,
) {
    let mut rotation_keyframes: Vec<(f32, Quat)> = Vec::with_capacity(curve.keyframes.len());

    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i) {
            if let Some(rotation) = keyframe.rotation {
                rotation_keyframes.push((timestamp, rotation.into()));
            }
        }
    }

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

fn process_scale_curve(clip: &mut AnimationClip, bone_name: &str, curve: &SerializableAnimCurve) {
    let mut scale_keyframes: Vec<(f32, Vec3)> = Vec::with_capacity(curve.keyframes.len());

    for (i, &timestamp) in curve.timestamps.iter().enumerate() {
        if let Some(keyframe) = curve.keyframes.get(i) {
            if let Some(scale) = keyframe.scale {
                scale_keyframes.push((timestamp, scale.into()));
            }
        }
    }

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
