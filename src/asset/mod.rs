pub mod animation;
pub mod locale;
pub mod material;
pub mod mesh;
pub mod model;
pub mod sound;
pub mod spawner;
pub mod sprite;
pub mod texture;
pub mod texture_atlas;

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct UInt2 {
    pub x: u32,
    pub y: u32,
}

impl From<UInt2> for UVec2 {
    fn from(val: UInt2) -> Self {
        UVec2::new(val.x, val.y)
    }
}

/// A serializable 4-component vector of `u16` values.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct UInt4 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub w: u16,
}

impl From<UInt4> for [u16; 4] {
    /// Converts a `UInt4` into a `[u16; 4]`.
    fn from(val: UInt4) -> Self {
        [val.x, val.y, val.z, val.w]
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Atlas {
    pub min: UInt2,
    pub max: UInt2,
}

impl From<Atlas> for URect {
    fn from(val: Atlas) -> Self {
        URect {
            min: val.min.into(),
            max: val.max.into(),
        }
    }
}

/// A serializable 2-component vector of `f32` values.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Float2 {
    pub x: f32,
    pub y: f32,
}

impl From<Float2> for Vec2 {
    /// Converts a `Float2` into a `Vec2`.
    fn from(val: Float2) -> Self {
        Vec2::new(val.x, val.y)
    }
}

/// A serializable 3-component vector of `f32` values.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Float3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Float3> for Vec3 {
    /// Converts a `Float3` into a `Vec3`.
    fn from(val: Float3) -> Self {
        Vec3::new(val.x, val.y, val.z)
    }
}

/// A serializable 4-component vector of `f32` values.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Float4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<Float4> for Vec4 {
    /// Converts a `Float4` into a `Vec4`.
    fn from(val: Float4) -> Self {
        Vec4::new(val.x, val.y, val.z, val.w)
    }
}

impl From<Float4> for Quat {
    /// Converts a `Float4` into a `Quat`.
    fn from(value: Float4) -> Self {
        Quat::from_vec4(value.into())
    }
}

/// A serializable 4x4 matrix of `f32` values.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Float4x4 {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m03: f32,

    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,

    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,

    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
}

impl From<Float4x4> for Mat4 {
    /// Converts a `Float4x4` into a `Mat4`.
    fn from(val: Float4x4) -> Self {
        Mat4::from_cols(
            Vec4::new(val.m00, val.m01, val.m02, val.m03),
            Vec4::new(val.m10, val.m11, val.m12, val.m13),
            Vec4::new(val.m20, val.m21, val.m22, val.m23),
            Vec4::new(val.m30, val.m31, val.m32, val.m33),
        )
    }
}
