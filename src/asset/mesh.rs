use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    math::{Vec2, Vec3, Vec4},
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Float2, Float3, Float4, Float4x4, UInt4};

/// A serializable representation of a mesh.
///
/// This struct is used to load mesh data from a file (e.g., JSON).
/// It can then be converted into a `MeshAsset` for use in the Bevy engine.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableMesh {
    /// The vertex positions of the mesh.
    pub positions: Vec<Float3>,
    /// The vertex colors of the mesh.
    pub colors: Vec<Float4>,
    /// The vertex UVs of the mesh.
    pub uvs: Vec<Float2>,
    /// The vertex normals of the mesh.
    pub normals: Vec<Float3>,
    // /// The vertex tangents of the mesh.
    // pub tangents: Vec<Float4>,
    /// The bone indices for each vertex.
    pub bone_indices: Vec<UInt4>,
    /// The bone weights for each vertex.
    pub bone_weights: Vec<Float4>,
    /// The submeshes of the mesh, each represented by a list of indices.
    pub submeshes: Vec<Vec<u32>>,
    /// The bind poses for the bones of the mesh.
    pub bindposes: Vec<Float4x4>,
    /// The names of the bones of the mesh.
    pub bones: Vec<String>,
}

/// A mesh asset.
///
/// This struct holds the data for a mesh that can be used in the Bevy engine.
#[derive(Asset, TypePath)]
pub struct MeshAsset {
    /// The serializable mesh data.
    pub serializable: SerializableMesh,
}

impl MeshAsset {
    /// Returns the vertex positions of the mesh as a `Vec<Vec3>`.
    pub fn positions(&self) -> Vec<Vec3> {
        self.serializable
            .positions
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }

    /// Returns the vertex colors of the mesh as a `Vec<Vec4>`.
    pub fn colors(&self) -> Vec<Vec4> {
        self.serializable
            .colors
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }

    /// Returns the vertex normals of the mesh as a `Vec<Vec3>`.
    pub fn normals(&self) -> Vec<Vec3> {
        self.serializable
            .normals
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }

    // /// Returns the vertex tangents of the mesh as a `Vec<Vec4>`.
    // pub fn tangents(&self) -> Vec<Vec4> {
    //     self.serializable
    //         .tangents
    //         .iter()
    //         .copied()
    //         .map(|v| v.into())
    //         .collect()
    // }

    /// Returns the vertex UVs of the mesh as a `Vec<Vec2>`.
    pub fn uvs(&self) -> Vec<Vec2> {
        self.serializable
            .uvs
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }

    /// Returns the bone indices for each vertex as a `Vec<[u16; 4]>`.
    pub fn bone_indices(&self) -> Vec<[u16; 4]> {
        self.serializable
            .bone_indices
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }

    /// Returns the bone weights for each vertex as a `Vec<Vec4>`.
    pub fn bone_weights(&self) -> Vec<Vec4> {
        self.serializable
            .bone_weights
            .iter()
            .copied()
            .map(|v| v.into())
            .collect()
    }
}

/// An error that can occur when loading a mesh.
#[derive(Debug, thiserror::Error)]
pub enum MeshLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for mesh assets.
#[derive(Default)]
pub struct MeshAssetLoader;

impl AssetLoader for MeshAssetLoader {
    type Asset = MeshAsset;
    type Settings = ();
    type Error = MeshLoaderError;

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
            // Deserialize the bytes into a `SerializableMesh`.
            let serializable: SerializableMesh = serde_json::from_slice(&bytes)?;

            // Create the `MeshAsset`.
            Ok(MeshAsset { serializable })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mesh"]
    }
}
