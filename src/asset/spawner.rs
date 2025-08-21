use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
};

use crate::asset::{
    animation::AnimationAssetLoader,
    material::MaterialAssetLoader,
    mesh::{MeshAsset, MeshAssetLoader},
    model::{ModelAsset, ModelAssetLoader, SerializableModelNode},
    texture::TexelAssetLoader,
};

/// A plugin that adds the custom asset loaders and the model spawning system.
pub struct CustomAssetPlugin;

impl Plugin for CustomAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ModelAsset>()
            .init_asset::<MeshAsset>()
            .register_asset_loader(ModelAssetLoader)
            .register_asset_loader(MeshAssetLoader)
            .register_asset_loader(MaterialAssetLoader)
            .register_asset_loader(TexelAssetLoader)
            .register_asset_loader(AnimationAssetLoader)
            .add_systems(Update, spawn_model_system);
    }
}

/// A component that marks an entity to have a model spawned as its child.
#[derive(Component)]
pub struct SpawnModel(pub Handle<ModelAsset>);

/// A system that spawns models.
///
/// This system looks for entities with the `SpawnModel` component and spawns the corresponding model.
fn spawn_model_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    models_to_spawn: Query<(Entity, &SpawnModel)>,
    model_assets: Res<Assets<ModelAsset>>,
    mesh_assets: Res<Assets<MeshAsset>>,
    material_assets: Res<Assets<StandardMaterial>>,
    mut inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    for (entity, spawn_request) in &models_to_spawn {
        let model_asset = match model_assets.get(&spawn_request.0) {
            Some(asset) => asset,
            None => continue,
        };

        // Wait for all meshes and materials to be loaded.
        let all_meshes_loaded = model_asset
            .meshes
            .values()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));
        if !all_meshes_loaded {
            continue;
        }

        let all_materials_loaded = model_asset
            .materials
            .values()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));
        if !all_materials_loaded {
            continue;
        }

        info!("Spawning model: {}", model_asset.serializable.root.name);

        // Spawn the model hierarchy.
        let mut nodes = HashMap::default();
        let root_entity =
            spawn_node_recursive(&mut commands, &mut nodes, &model_asset.serializable.root);

        // Add the render components to the model hierarchy.
        add_render_components_recursive(
            &mut commands,
            &model_asset.serializable.root,
            &nodes,
            &model_asset,
            &mesh_assets,
            &material_assets,
            &mut inverse_bindposes_assets,
        );

        // Add the model to the entity that requested it.
        commands
            .entity(entity)
            .add_child(root_entity)
            .remove::<SpawnModel>();
    }
}

/// Recursively spawns the nodes of a model hierarchy.
fn spawn_node_recursive(
    commands: &mut Commands,
    nodes: &mut HashMap<String, Entity>,
    node: &SerializableModelNode,
) -> Entity {
    let children: Vec<_> = node
        .children
        .iter()
        .map(|child| spawn_node_recursive(commands, nodes, child))
        .collect();

    let mut entity_commands = commands.spawn((
        Name::new(node.name.clone()),
        Transform::from_matrix(node.transform.into()),
    ));
    entity_commands.add_children(&children);

    nodes.insert(node.name.clone(), entity_commands.id());
    entity_commands.id()
}

/// Recursively adds the render components to the model hierarchy.
/// This function now directly uses the handles to Bevy's native `Mesh` and `StandardMaterial` assets.
fn add_render_components_recursive(
    commands: &mut Commands,
    node: &SerializableModelNode,
    nodes: &HashMap<String, Entity>,
    model_asset: &ModelAsset,
    mesh_assets: &Res<Assets<MeshAsset>>,
    material_assets: &Res<Assets<StandardMaterial>>,
    inverse_bindposes_assets: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    if let Some(mesh_uri) = &node.mesh {
        let mesh_asset_handle = model_asset.meshes.get(mesh_uri).unwrap();
        let mesh_asset = mesh_assets.get(mesh_asset_handle).unwrap();

        let is_skinned = !mesh_asset.bones.is_empty() && !mesh_asset.bindposes.is_empty();
        let mut skinned_mesh_component = None;

        if is_skinned {
            let joints: Vec<_> = mesh_asset
                .bones
                .iter()
                .filter_map(|bone_name| match nodes.get(bone_name) {
                    Some(entity) => Some(*entity),
                    None => {
                        error!("Bone entity not found!");
                        None
                    }
                })
                .collect();

            let inverse_bindpose_handle = inverse_bindposes_assets.add(
                SkinnedMeshInverseBindposes::from(mesh_asset.bindposes.clone()),
            );

            skinned_mesh_component = Some(SkinnedMesh {
                inverse_bindposes: inverse_bindpose_handle,
                joints,
            });
        }

        for (i, material_uri) in node.materials.iter().enumerate() {
            let pair = (
                mesh_asset.submeshes.get(i),
                model_asset.materials.get(material_uri),
            );
            if let (Some(submesh), Some(material_handle)) = pair {
                let mut render_entity_commands = commands.spawn((
                    Mesh3d(submesh.clone()),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::IDENTITY,
                ));

                if let Some(skinned_mesh_component) = skinned_mesh_component.as_ref() {
                    render_entity_commands.insert(skinned_mesh_component.clone());
                }

                let render_entity = render_entity_commands.id();
                let parent_entity = nodes.get(&node.name).copied().unwrap();
                commands.entity(parent_entity).add_child(render_entity);
            }
        }
    }

    for child in &node.children {
        add_render_components_recursive(
            commands,
            child,
            nodes,
            model_asset,
            mesh_assets,
            material_assets,
            inverse_bindposes_assets,
        );
    }
}
