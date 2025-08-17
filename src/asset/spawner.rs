use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageFormat, ImageType},
    prelude::*,
    render::mesh::{
        Indices, PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
};

use crate::asset::{
    material::{MaterialAsset, MaterialAssetLoader},
    mesh::{MeshAsset, MeshAssetLoader},
    model::{ModelAsset, ModelAssetLoader, SerializableModelNode},
    texture::{TexelAsset, TexelAssetLoader},
};

/// A resource to cache converted meshes and materials.
#[derive(Default, Resource)]
pub struct ConvertedAssetCache {
    /// A map from mesh URIs to their converted `Mesh` handles.
    pub meshes: HashMap<String, Handle<Mesh>>,
    pub materials: HashMap<Handle<MaterialAsset>, Handle<StandardMaterial>>,
    pub textures: HashMap<Handle<TexelAsset>, Handle<Image>>,
}

/// A plugin that adds the custom asset loaders and the model spawning system.
pub struct CustomAssetPlugin;

impl Plugin for CustomAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ModelAsset>()
            .init_asset::<MeshAsset>()
            .init_asset::<MaterialAsset>()
            .init_asset::<TexelAsset>()
            .init_resource::<ConvertedAssetCache>()
            .register_asset_loader(ModelAssetLoader)
            .register_asset_loader(MeshAssetLoader)
            .register_asset_loader(MaterialAssetLoader)
            .register_asset_loader(TexelAssetLoader)
            .add_systems(Update, spawn_model_system);
    }
}

/// A component that marks an entity to have a model spawned as its child.
#[derive(Component)]
pub struct SpawnModel(pub Handle<ModelAsset>);

/// A system that spawns models.
///
/// This system looks for entities with the `SpawnModel` component and spawns the corresponding model.
#[allow(clippy::too_many_arguments)]
fn spawn_model_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    models_to_spawn: Query<(Entity, &SpawnModel)>,
    model_assets: Res<Assets<ModelAsset>>,
    mesh_assets: Res<Assets<MeshAsset>>,
    material_assets: Res<Assets<MaterialAsset>>,
    texel_assets: Res<Assets<TexelAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut converted_asset_cache: ResMut<ConvertedAssetCache>,
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

        // Convert the custom mesh and material assets to Bevy's native assets.
        let converted_meshes = convert_meshes(
            model_asset.meshes.iter(),
            &mesh_assets,
            &mut meshes,
            &mut converted_asset_cache,
        );

        let converted_materials = convert_materials(
            model_asset.materials.values(),
            &material_assets,
            &texel_assets,
            &mut materials,
            &mut images,
            &mut converted_asset_cache,
        );

        // Spawn the model hierarchy.
        let mut nodes = HashMap::default();
        let root_entity =
            spawn_node_recursive(&mut commands, &mut nodes, &model_asset.serializable.root);

        // Add the render components to the model hierarchy.
        add_render_components_recursive(
            &mut commands,
            &model_asset.serializable.root,
            &nodes,
            model_asset,
            &mesh_assets,
            &converted_meshes,
            &converted_materials,
            &mut inverse_bindposes,
        );

        // Add the model to the entity that requested it.
        commands
            .entity(entity)
            .add_child(root_entity)
            .remove::<SpawnModel>();
    }
}

/// Converts custom mesh assets to Bevy's native `Mesh` assets.
fn convert_meshes<'a, I>(
    meshes_to_convert: I,
    mesh_assets: &Res<Assets<MeshAsset>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    cache: &mut ResMut<ConvertedAssetCache>,
) -> HashMap<String, Handle<Mesh>>
where
    I: Iterator<Item = (&'a String, &'a Handle<MeshAsset>)>,
{
    let mut converted_meshes = HashMap::default();
    for (uri, handle) in meshes_to_convert {
        let mesh_asset = mesh_assets.get(handle).unwrap();
        let submeshes = mesh_asset
            .serializable
            .submeshes
            .iter()
            .cloned()
            .enumerate();
        for (i, indices) in submeshes {
            let mesh_uri = format!("{}_{}", uri, i);
            match cache.meshes.get(&mesh_uri) {
                Some(mesh_handle) => {
                    converted_meshes.insert(mesh_uri, mesh_handle.clone());
                }
                None => {
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    );

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_asset.positions());
                    if !mesh_asset.colors().is_empty() {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_asset.colors());
                    }
                    if !mesh_asset.uvs().is_empty() {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_asset.uvs());
                    }
                    if !mesh_asset.normals().is_empty() {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_asset.normals());
                    }
                    if !mesh_asset.tangents().is_empty() {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, mesh_asset.tangents());
                    }
                    if !mesh_asset.bone_indices().is_empty() {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_JOINT_INDEX,
                            VertexAttributeValues::Uint16x4(mesh_asset.bone_indices()),
                        );
                    }
                    if !mesh_asset.bone_weights().is_empty() {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_JOINT_WEIGHT,
                            mesh_asset.bone_weights(),
                        );
                    }

                    mesh.insert_indices(Indices::U32(indices));
                    converted_meshes.insert(mesh_uri, meshes.add(mesh));
                }
            }
        }
    }

    converted_meshes
}

/// Converts custom material assets to Bevy's native `StandardMaterial` assets.
fn convert_materials<'a, I>(
    materials_to_convert: I,
    material_assets: &Res<Assets<MaterialAsset>>,
    texel_assets: &Res<Assets<TexelAsset>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    cache: &mut ResMut<ConvertedAssetCache>,
) -> HashMap<Handle<MaterialAsset>, Handle<StandardMaterial>>
where
    I: Iterator<Item = &'a Handle<MaterialAsset>>,
{
    let mut converted_materials = HashMap::default();
    for handle in materials_to_convert {
        match cache.materials.get(handle) {
            Some(material_handle) => {
                converted_materials.insert(handle.clone(), material_handle.clone());
            }
            None => {
                let material_asset = material_assets.get(handle).unwrap();
                let standard_material = StandardMaterial {
                    // This assumes that the base color texture is in sRGB format.
                    base_color_texture: material_asset.base_color_texture.as_ref().map(
                        |texel_to_convert| {
                            convert_texel(texel_to_convert, texel_assets, images, cache, true)
                        },
                    ),
                    metallic: material_asset.metallic,
                    perceptual_roughness: material_asset.roughness,
                    ..Default::default()
                };
                converted_materials.insert(handle.clone(), materials.add(standard_material));
            }
        }
    }

    converted_materials
}

/// Converts a `TexelAsset` to a Bevy `Image`.
///
/// This function takes a handle to a `TexelAsset`, which contains raw, possibly compressed
/// texture data. It then creates a Bevy `Image` from this data, assuming the KTX2 format.
/// The converted image is cached to avoid redundant conversions.
///
/// # Arguments
///
/// * `texel_to_convert` - A handle to the `TexelAsset` to convert.
/// * `texel_assets` - The `Assets` resource for `TexelAsset`.
/// * `images` - The `Assets` resource for `Image`.
/// * `cache` - The cache for converted assets.
/// * `is_srgb` - Whether the texture data is in sRGB format.
///
/// # Returns
///
/// A handle to the converted `Image`.
fn convert_texel(
    texel_to_convert: &Handle<TexelAsset>,
    texel_assets: &Res<Assets<TexelAsset>>,
    images: &mut ResMut<Assets<Image>>,
    cache: &mut ResMut<ConvertedAssetCache>,
    is_srgb: bool,
) -> Handle<Image> {
    match cache.textures.get(texel_to_convert) {
        Some(image_handle) => image_handle.clone(),
        None => {
            let texel_asset = texel_assets.get(texel_to_convert).unwrap();
            let image = Image::from_buffer(
                &texel_asset.data,
                ImageType::Format(ImageFormat::Ktx2),
                CompressedImageFormats::all(),
                is_srgb,
                bevy::image::ImageSampler::Default,
                RenderAssetUsages::RENDER_WORLD,
            )
            .unwrap();

            let image_handle = images.add(image);
            cache
                .textures
                .insert(texel_to_convert.clone(), image_handle.clone());

            image_handle
        }
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
#[allow(clippy::too_many_arguments)]
fn add_render_components_recursive(
    commands: &mut Commands,
    node: &SerializableModelNode,
    nodes: &HashMap<String, Entity>,
    model_asset: &ModelAsset,
    mesh_assets: &Res<Assets<MeshAsset>>,
    converted_meshes: &HashMap<String, Handle<Mesh>>,
    converted_materials: &HashMap<Handle<MaterialAsset>, Handle<StandardMaterial>>,
    inverse_bindposes_assets: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    if let Some(mesh_uri) = &node.mesh {
        let mesh_asset_handle = model_asset.meshes.get(mesh_uri).unwrap();
        let mesh_asset = mesh_assets.get(mesh_asset_handle).unwrap();

        let is_skinned = !mesh_asset.serializable.bones.is_empty();
        let mut skinned_mesh_component = None;

        if is_skinned {
            let joints: Vec<_> = mesh_asset
                .serializable
                .bones
                .iter()
                .map(|bone_name| {
                    nodes
                        .get(bone_name)
                        .copied()
                        .expect("Bone entity not found!")
                })
                .collect();

            let inverse_bindposes: Vec<Mat4> = mesh_asset
                .serializable
                .bindposes
                .iter()
                .copied()
                .map(|m| m.into())
                .collect();
            let inverse_bindpose_handle =
                inverse_bindposes_assets.add(SkinnedMeshInverseBindposes::from(inverse_bindposes));

            skinned_mesh_component = Some(SkinnedMesh {
                inverse_bindposes: inverse_bindpose_handle,
                joints,
            });
        }

        for (i, material_uri) in node.materials.iter().enumerate() {
            let submesh_uri = format!("{}_{}", mesh_uri, i);
            let material_handle = model_asset.materials.get(material_uri).unwrap();
            let pair = (
                converted_meshes.get(&submesh_uri),
                converted_materials.get(material_handle),
            );

            if let (Some(mesh_handle), Some(material_handle)) = pair {
                let mut render_entity_commands = commands.spawn((
                    Mesh3d(mesh_handle.clone()),
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
            converted_meshes,
            converted_materials,
            inverse_bindposes_assets,
        );
    }
}
