use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
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
    /// The vertex tangents of the mesh.
    pub tangents: Vec<Float4>,
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

#[derive(Asset, TypePath)]
pub struct MeshAsset {
    pub bones: Vec<String>,
    pub bindposes: Vec<Mat4>,
    pub submeshes: Vec<Handle<Mesh>>,
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
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        debug!("asset load: {}", &load_context.asset_path());
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 데이터 복호화
            // TODO: 데이터 압축 해제
            // Deserialize the bytes into a `SerializableMesh`.
            let serializable: SerializableMesh = serde_json::from_slice(&bytes)?;

            // Create a base mesh with all vertex attributes.
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

            // Create a separate mesh for each submesh, with its own indices.
            let submeshes: Vec<_> = serializable
                .submeshes
                .iter()
                .enumerate()
                .map(|(i, indices)| {
                    let mut submesh = mesh.clone();
                    submesh.insert_indices(Indices::U32(indices.clone()));

                    // Add the submesh as a labeled asset to the load context.
                    let label = format!("{}_{i}", &load_context.asset_path());
                    let mesh_handle: Handle<Mesh> = load_context.add_labeled_asset(label, submesh);

                    mesh_handle
                })
                .collect();

            let bones = serializable.bones.clone();
            let bindposes = serializable
                .bindposes
                .iter()
                .cloned()
                .map(|m| m.into())
                .collect();

            // Create the `MeshAsset` with the bone data and submesh handles.
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
