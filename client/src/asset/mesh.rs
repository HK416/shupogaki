use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Float2, Float3, Float4, Float4x4, UInt4};

use super::*;

/// A serializable representation of a mesh, designed for loading from a file.
///
/// This struct contains all the raw vertex data for a mesh, including skinning information.
/// It is converted into a `MeshAsset` during the loading process.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableMesh {
    /// The vertex positions (x, y, z).
    pub positions: Vec<Float3>,
    /// The vertex colors (r, g, b, a).
    pub colors: Vec<Float4>,
    /// The vertex UV coordinates (u, v).
    pub uvs: Vec<Float2>,
    /// The vertex normals (x, y, z).
    pub normals: Vec<Float3>,
    /// The vertex tangents (x, y, z, w).
    pub tangents: Vec<Float4>,
    /// The bone indices for each vertex, used for skinning.
    pub bone_indices: Vec<UInt4>,
    /// The bone weights for each vertex, used for skinning.
    pub bone_weights: Vec<Float4>,
    /// A list of submeshes, where each submesh is a list of vertex indices.
    pub submeshes: Vec<Vec<u32>>,
    /// The inverse bind pose matrices for each bone in the skeleton.
    pub bindposes: Vec<Float4x4>,
    /// The names of the bones in the skeleton.
    pub bones: Vec<String>,
}

// Helper methods to convert the serializable vector types into Bevy-compatible slices.
impl SerializableMesh {
    pub fn positions(&self) -> Vec<[f32; 3]> {
        self.positions.iter().map(|v| [v.x, v.y, v.z]).collect()
    }

    pub fn colors(&self) -> Vec<[f32; 4]> {
        self.colors.iter().map(|v| [v.x, v.y, v.z, v.w]).collect()
    }

    pub fn uvs(&self) -> Vec<[f32; 2]> {
        self.uvs.iter().map(|v| [v.x, v.y]).collect()
    }

    pub fn normals(&self) -> Vec<[f32; 3]> {
        self.normals.iter().map(|v| [v.x, v.y, v.z]).collect()
    }

    pub fn tangents(&self) -> Vec<[f32; 4]> {
        self.tangents.iter().map(|v| [v.x, v.y, v.z, v.w]).collect()
    }

    pub fn bone_indices(&self) -> Vec<[u16; 4]> {
        self.bone_indices
            .iter()
            .map(|v| [v.x, v.y, v.z, v.w])
            .collect()
    }

    pub fn bone_weights(&self) -> Vec<[f32; 4]> {
        self.bone_weights
            .iter()
            .map(|v| [v.x, v.y, v.z, v.w])
            .collect()
    }
}

/// A custom mesh asset that holds bone information and handles to its submeshes.
///
/// A single `.mesh` file can contain multiple submeshes, each with a different material.
/// This asset structure allows us to manage them together.
#[derive(Asset, TypePath)]
pub struct MeshAsset {
    /// The names of the bones in the mesh's skeleton.
    pub bones: Vec<String>,
    /// The inverse bind pose matrices for each bone.
    pub bindposes: Vec<Mat4>,
    /// A list of handles to the individual Bevy `Mesh` assets for each submesh.
    pub submeshes: Vec<Handle<Mesh>>,
}

/// An error that can occur when loading a `.mesh` asset.
#[derive(Debug, thiserror::Error)]
pub enum MeshLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to decrypt asset for the following reason:{0}")]
    Crypt(#[from] anyhow::Error),
}

/// A loader for `.mesh` assets.
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
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        info!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the raw bytes from the asset file.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let key = reconstruct_key();
            let decrypted_data = decrypt_bytes(&bytes, &key)?;

            // Deserialize the bytes from JSON into a `SerializableMesh`.
            let serializable: SerializableMesh = serde_json::from_slice(&decrypted_data)?;

            // Create a base Bevy `Mesh` and insert all the vertex attributes from the serializable data.
            // This base mesh contains the vertex data for all submeshes.
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            );

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, serializable.positions());
            if !serializable.colors.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, serializable.colors());
            }
            if !serializable.uvs.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, serializable.uvs());
            }
            if !serializable.normals.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, serializable.normals());
            }
            if !serializable.tangents.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, serializable.tangents());
            }
            if !serializable.bone_indices.is_empty() {
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_INDEX,
                    VertexAttributeValues::Uint16x4(serializable.bone_indices()),
                );
            }
            if !serializable.bone_weights.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, serializable.bone_weights());
            }

            // For each submesh defined in the serializable data, create a new Bevy `Mesh`.
            let submeshes: Vec<_> = serializable
                .submeshes
                .iter()
                .enumerate()
                .map(|(i, indices)| {
                    // Clone the base mesh with all its vertex attributes.
                    let mut submesh = mesh.clone();
                    // Set the indices for this specific submesh.
                    submesh.insert_indices(Indices::U32(indices.clone()));

                    // Add the submesh as a labeled asset to the load context.
                    // This makes it a dependency of the main `MeshAsset`.
                    let label = format!("{}_{i}", &load_context.asset_path());
                    let mesh_handle: Handle<Mesh> = load_context.add_labeled_asset(label, submesh);

                    mesh_handle
                })
                .collect();

            // Collect bone names and bind poses.
            let bones = serializable.bones.clone();
            let bindposes = serializable
                .bindposes
                .iter()
                .cloned()
                .map(|m| m.into())
                .collect();

            // Create the final `MeshAsset` with the bone data and handles to the submeshes.
            Ok(MeshAsset {
                bones,
                bindposes,
                submeshes,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mesh"]
    }
}
