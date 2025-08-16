use std::collections::HashMap;

use bevy::{
    asset::{Asset, AssetLoader, Handle, LoadContext, io::Reader},
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use serde::Deserialize;

use crate::asset::{Float4x4, material::MaterialAsset, mesh::MeshAsset};

/// A serializable representation of a model.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableModel {
    /// The root node of the model's hierarchy.
    pub root: SerializableModelNode,
}

/// A serializable representation of a node in a model's hierarchy.
#[derive(Debug, Deserialize, Clone)]
pub struct SerializableModelNode {
    /// The name of the node.
    pub name: String,
    /// The transform of the node.
    pub transform: Float4x4,
    /// The mesh used by this node, if any.
    pub mesh: Option<String>,
    /// The materials used by this node.
    pub materials: Vec<String>,
    /// The children of this node.
    pub children: Vec<SerializableModelNode>,
}

/// A model asset.
#[derive(Asset, TypePath)]
pub struct ModelAsset {
    /// The serializable model data.
    pub serializable: SerializableModel,
    /// A map of mesh names to their handles.
    pub meshes: HashMap<String, Handle<MeshAsset>>,
    /// A map of material names to their handles.
    pub materials: HashMap<String, Handle<MaterialAsset>>,
}

/// An error that can occur when loading a model.
#[derive(Debug, thiserror::Error)]
pub enum ModelLoaderError {
    /// An I/O error occurred.
    #[error("Failed to load asset for the following reason:{0}")]
    IO(#[from] std::io::Error),
    /// A JSON deserialization error occurred.
    #[error("Failed to decode asset for the following reason:{0}")]
    Json(#[from] serde_json::Error),
}

/// A loader for model assets.
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
        Box::pin(async move {
            // Read the bytes from the reader.
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            // TODO: 복호화
            // TODO: 압축 해제
            // Deserialize the bytes into a `SerializableModel`.
            let serializable: SerializableModel = serde_json::from_slice(&bytes)?;

            // Collect all the meshes and materials used by the model.
            let mut meshes_to_load = HashMap::default();
            let mut material_to_load = HashMap::default();
            collect_assets_recursive(
                &serializable.root,
                &mut meshes_to_load,
                &mut material_to_load,
                load_context,
            );

            // Create the `ModelAsset`.
            Ok(ModelAsset {
                serializable,
                meshes: meshes_to_load,
                materials: material_to_load,
            })
        })
    }
}

/// Recursively collects all the meshes and materials used by a model.
fn collect_assets_recursive(
    node: &SerializableModelNode,
    meshes: &mut HashMap<String, Handle<MeshAsset>>,
    materials: &mut HashMap<String, Handle<MaterialAsset>>,
    load_context: &mut LoadContext,
) {
    // Load the mesh for the current node, if it exists.
    if let Some(mesh_uri) = &node.mesh {
        let mesh_path = format!("{}.mesh", mesh_uri);
        let handle: Handle<MeshAsset> = load_context.load(mesh_path);
        meshes.insert(mesh_uri.clone(), handle);
    }

    // Load the materials for the current node.
    for material_uri in &node.materials {
        let material_path = format!("{}.material", material_uri);
        let handle: Handle<MaterialAsset> = load_context.load(material_path);
        materials.insert(material_uri.clone(), handle);
    }

    // Recursively collect the assets for the children of the current node.
    for child in &node.children {
        collect_assets_recursive(child, meshes, materials, load_context);
    }
}
