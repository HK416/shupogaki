#![allow(dead_code)]
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

#[derive(Component)]
pub struct EyeMouth(pub Handle<ExtendedMaterial<StandardMaterial, FacialExpressionExtension>>);

#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct FacialExpressionUniform {
    pub index: UVec4,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct FacialExpressionExtension {
    #[texture(100)]
    #[sampler(101)]
    pub mouth_atlas: Handle<Image>,

    #[uniform(102)]
    pub uniform: FacialExpressionUniform,
}

impl MaterialExtension for FacialExpressionExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/face_mouth.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/face_mouth.wgsl".into()
    }
}
