use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    platform::collections::HashMap,
    prelude::*,
    render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
};

use crate::{
    asset::{
        animation::AnimationAssetLoader,
        config::{ConfigAssetLoader, Configuration},
        locale::{CurrentLocale, LocalizationAssets, LocalizationData, LocalizationDataLoader},
        material::{FaceMouthMaterialAssetLoader, MaterialAssetLoader},
        mesh::{MeshAsset, MeshAssetLoader},
        model::{MaterialHandle, ModelAsset, ModelAssetLoader, SerializableModelNode},
        sound::SoundAssetLoader,
        sprite::SpriteAssetLoader,
        texture::TexelAssetLoader,
        texture_atlas::TextureAtlasAssetLoader,
    },
    shader::face_mouth::EyeMouth,
};

/// A plugin that adds the custom asset loaders and the model spawning system.
pub struct CustomAssetPlugin;

impl Plugin for CustomAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ModelAsset>()
            .init_asset::<MeshAsset>()
            .init_asset::<Configuration>()
            .init_asset::<LocalizationData>()
            .init_resource::<CurrentLocale>()
            .register_asset_loader(ModelAssetLoader)
            .register_asset_loader(MeshAssetLoader)
            .register_asset_loader(MaterialAssetLoader)
            .register_asset_loader(FaceMouthMaterialAssetLoader)
            .register_asset_loader(TextureAtlasAssetLoader)
            .register_asset_loader(TexelAssetLoader)
            .register_asset_loader(SpriteAssetLoader)
            .register_asset_loader(AnimationAssetLoader)
            .register_asset_loader(LocalizationDataLoader)
            .register_asset_loader(SoundAssetLoader)
            .register_asset_loader(ConfigAssetLoader)
            .add_systems(
                Update,
                (
                    spawn_model_system,
                    changed_translation_system.run_if(resource_changed::<CurrentLocale>),
                    added_translation_system,
                ),
            );
    }
}

#[derive(Component)]
pub struct TranslatableText(pub String);

fn changed_translation_system(
    locale: Res<CurrentLocale>,
    localization_assets: Res<LocalizationAssets>,
    localization_data: Res<Assets<LocalizationData>>,
    mut query: Query<(&mut Text, &TranslatableText)>,
) {
    if let Some(locale_data) = localization_assets.locale.get(&locale.0)
        && let Some(translations) = localization_data.get(locale_data.id())
    {
        for (mut text, translatable_text) in query.iter_mut() {
            if let Some(translation) = translations.0.get(&translatable_text.0) {
                *text = Text::new(translation);
            } else {
                error!("Translation not found: {}", translatable_text.0);
            }
        }
    }
}

fn added_translation_system(
    locale: Res<CurrentLocale>,
    localization_assets: Res<LocalizationAssets>,
    localization_data: Res<Assets<LocalizationData>>,
    mut query: Query<(&mut Text, &TranslatableText), Added<TranslatableText>>,
) {
    if let Some(locale_data) = localization_assets.locale.get(&locale.0)
        && let Some(translations) = localization_data.get(locale_data.id())
    {
        for (mut text, translatable_text) in query.iter_mut() {
            if let Some(translation) = translations.0.get(&translatable_text.0) {
                *text = Text::new(translation);
            } else {
                error!("Translation not found: {}", translatable_text.0);
            }
        }
    }
}

/// A component that marks an entity to have a model spawned as its child.
#[derive(Component)]
pub struct SpawnModel(pub Handle<ModelAsset>);

/// A system that spawns models.
///
/// This system looks for entities with the `SpawnModel` component and spawns the corresponding model.
/// Spawns the models that have been requested to be spawned.
///
/// The spawning process is divided into two main steps to correctly handle skinned meshes and animations:
/// 1. **Logical Hierarchy Spawning**: `spawn_node_recursive` creates a hierarchy of entities with `Transform`
///    and `AnimationTarget` components. This hierarchy mirrors the bone structure and is essential for
///    the animation system to find its targets by name.
/// 2. **Visual Component Attachment**: `add_render_components_recursive` traverses the newly created logical
///    hierarchy and attaches the visible components (`Mesh3d`, `MeshMaterial3d`, `SkinnedMesh`) as children
///    to the appropriate logical nodes. This ensures that `SkinnedMesh` can find the `Entity` IDs of its
///    joints (bones) from the logical hierarchy.
fn spawn_model_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    models_to_spawn: Query<(Entity, &SpawnModel)>,
    model_assets: Res<Assets<ModelAsset>>,
    mesh_assets: Res<Assets<MeshAsset>>,
    mut inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    for (root_entity, spawn_request) in &models_to_spawn {
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

        let all_materials_loaded = model_asset.materials.values().all(|handle| match handle {
            MaterialHandle::Standard(handle) => {
                asset_server.is_loaded_with_dependencies(handle.id())
            }
            MaterialHandle::FacialExpression(handle) => {
                asset_server.is_loaded_with_dependencies(handle.id())
            }
        });
        if !all_materials_loaded {
            continue;
        }

        info!("Spawning model: {}", model_asset.serializable.root.name);

        // Step 1: Spawn the logical model hierarchy for animation targeting.
        let mut nodes = HashMap::default();
        let entity = spawn_node_recursive(
            root_entity, // The entity that the animation player is attached to.
            &mut commands,
            &mut nodes,
            &model_asset.serializable.root,
        );

        // Step 2: Add the render components (meshes, materials) to the logical hierarchy.
        add_render_components_recursive(
            &mut commands,
            &model_asset.serializable.root,
            &nodes,
            model_asset,
            &mesh_assets,
            &mut inverse_bindposes_assets,
        );

        // Add the fully constructed model as a child to the entity that requested it.
        commands
            .entity(root_entity)
            .add_child(entity)
            .remove::<SpawnModel>();
    }
}

/// Recursively spawns the logical hierarchy of a model.
///
/// This function creates entities with `Transform` and `AnimationTarget` components,
/// establishing the parent-child relationships and names needed for animation targeting.
/// It does not add any visible components.
fn spawn_node_recursive(
    root_entity: Entity, // The entity that the animation player is attached to.
    commands: &mut Commands,
    nodes: &mut HashMap<String, Entity>,
    node: &SerializableModelNode,
) -> Entity {
    let children: Vec<_> = node
        .children
        .iter()
        .map(|child| spawn_node_recursive(root_entity, commands, nodes, child))
        .collect();

    let mut entity_commands = commands.spawn((
        // The animation player is attached to the root entity, so we need to tell it which
        // entity to play the animation on.
        AnimationTarget {
            id: AnimationTargetId::from_name(&Name::new(node.name.clone())),
            player: root_entity,
        },
        Transform::from_matrix(node.transform.into()),
        Visibility::Inherited,
    ));
    entity_commands.add_children(&children);

    nodes.insert(node.name.clone(), entity_commands.id());
    entity_commands.id()
}

/// Recursively adds the render components (meshes, materials) to the logical model hierarchy.
///
/// This function traverses the hierarchy and, for nodes that have a mesh, spawns child entities
/// with the actual `Mesh3d` and `MeshMaterial3d` components. It also adds the `SkinnedMesh`
/// component if the mesh is skinned, linking it to the joint entities created in `spawn_node_recursive`.
fn add_render_components_recursive(
    commands: &mut Commands,
    node: &SerializableModelNode,
    nodes: &HashMap<String, Entity>,
    model_asset: &ModelAsset,
    mesh_assets: &Res<Assets<MeshAsset>>,
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
                let mut render_entity_commands = match material_handle {
                    MaterialHandle::Standard(handle) => commands.spawn((
                        Mesh3d(submesh.clone()),
                        MeshMaterial3d(handle.clone()),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                    )),
                    MaterialHandle::FacialExpression(handle) => commands.spawn((
                        Mesh3d(submesh.clone()),
                        MeshMaterial3d(handle.clone()),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                        EyeMouth(handle.clone()),
                    )),
                };

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
            inverse_bindposes_assets,
        );
    }
}
