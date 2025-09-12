use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    pbr::ExtendedMaterial,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::{
    asset::{Float4x4, mesh::MeshAsset},
    shader::face_mouth::FacialExpressionExtension,
};

/// A serializable representation of a model's hierarchy, designed for loading from a file.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableModel {
    /// The root node of the model's hierarchy.
    pub root: SerializableModelNode,
}

/// A serializable representation of a single node within a model's hierarchy.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableModelNode {
    /// The name of the node, used for identification (e.g., for animation targeting).
    pub name: String,
    /// The local transform of this node relative to its parent.
    pub transform: Float4x4,
    /// The name of the mesh used by this node, if any.
    pub mesh: Option<String>,
    /// A list of material names used by this node's mesh.
    pub materials: Vec<String>,
    /// The children of this node in the hierarchy.
    pub children: Vec<SerializableModelNode>,
}

pub enum MaterialHandle {
    Standard(Handle<StandardMaterial>),
    FacialExpression(Handle<ExtendedMaterial<StandardMaterial, FacialExpressionExtension>>),
}

/// A custom model asset that holds the model hierarchy and handles to its dependencies.
///
/// This asset is the result of loading a `.hierarchy` file. It contains the deserialized
/// node structure and handles to all the `MeshAsset` and `StandardMaterial` assets
/// that the model requires.
#[derive(Asset, TypePath)]
pub struct ModelAsset {
    /// The deserialized model hierarchy data.
    pub serializable: SerializableModel,
    /// A map of mesh names to their loaded `MeshAsset` handles.
    pub meshes: HashMap<String, Handle<MeshAsset>>,
    /// A map of material names to their loaded `StandardMaterial` handles.
    pub materials: HashMap<String, MaterialHandle>,
}

/// An error that can occur when loading a `.hierarchy` asset.
#[derive(Debug, thiserror::Error)]
pub enum ModelLoaderError {
    /// An I/O error occurred while reading the asset file.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for `.hierarchy` assets.
#[derive(Default)]
pub struct ModelAssetLoader;

impl AssetLoader for ModelAssetLoader {
    type Asset = ModelAsset;
    type Settings = ();
    type Error = ModelLoaderError;

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

            // TODO: 복호화 (Decryption)
            // TODO: 압축 해제 (Decompression)

            // Deserialize the bytes from JSON into a `SerializableModel`.
            let serializable: SerializableModel = serde_json::from_slice(&bytes)?;

            // Recursively traverse the model hierarchy to find all mesh and material dependencies.
            let mut meshes_to_load = HashMap::default();
            let mut material_to_load = HashMap::default();
            collect_assets_recursive(
                &serializable.root,
                &mut meshes_to_load,
                &mut material_to_load,
                load_context,
            );

            // Create the final `ModelAsset`, which holds the hierarchy and the handles to its dependencies.
            Ok(ModelAsset {
                serializable,
                meshes: meshes_to_load,
                materials: material_to_load,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hierarchy"]
    }
}

/// Recursively traverses the model hierarchy to collect and load all mesh and material assets.
fn collect_assets_recursive(
    node: &SerializableModelNode,
    meshes: &mut HashMap<String, Handle<MeshAsset>>,
    materials: &mut HashMap<String, MaterialHandle>,
    load_context: &mut LoadContext,
) {
    // If the node has a mesh, load the corresponding `.mesh` asset.
    if let Some(mesh_uri) = &node.mesh {
        let mesh_path = format!("meshes/{}.mesh", mesh_uri);
        let handle: Handle<MeshAsset> = load_context.load(mesh_path);
        meshes.insert(mesh_uri.clone(), handle);
    }

    // Load all materials associated with the node.
    for material_uri in &node.materials {
        let handle: MaterialHandle = match material_uri.contains("EyeMouth") {
            true => {
                let material_path = format!("materials/{}.material", material_uri);
                let handle: Handle<ExtendedMaterial<StandardMaterial, FacialExpressionExtension>> =
                    load_context.load(material_path);
                MaterialHandle::FacialExpression(handle)
            }
            false => {
                let material_path = format!("materials/{}.material", material_uri);
                let handle: Handle<StandardMaterial> = load_context.load(material_path);
                MaterialHandle::Standard(handle)
            }
        };
        materials.insert(material_uri.clone(), handle);
    }

    // Recurse into child nodes.
    for child in &node.children {
        collect_assets_recursive(child, meshes, materials, load_context);
    }
}
