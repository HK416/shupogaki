// Import necessary Bevy modules.
use bevy::{prelude::*, render::view::NoFrustumCulling};

use super::*;

// --- SETUP SYSTEM ---

pub fn on_enter(mut commands: Commands) {
    info!("Enter ResultStart State.");
    // --- Resource initialization ---
    commands.insert_resource(SceneTimer::default());

    // --- Lighting ---
    // Spawn a directional light to illuminate the scene.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0, // A bright, sun-like light.
            shadows_enabled: true, // Enable shadows for realism.
            ..Default::default()
        },
        Transform::from_xyz(8.0, 12.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // --- Camera Spawn ---
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 52f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        }),
        Transform::from_xyz(3.0, 3.0, 0.0).looking_at((0.0, 1.8, 0.0).into(), Vec3::Y),
    ));
}

pub fn spawn_ground(mut commands: Commands, cached: Res<CachedGrounds>) {
    let model = cached.models.get(&GroundModel::Plane999).unwrap();
    commands.spawn((
        SpawnModel(model.clone()),
        Transform::IDENTITY,
        InGameStateEntity,
    ));
}

pub fn enable_result_ui(mut query: Query<(&mut Visibility, &UI)>) {
    for (mut visibility, ui) in query.iter_mut() {
        match *ui {
            _ => *visibility = Visibility::Hidden,
        }
    }
}

// --- UPDATE SYSTEMS ---

pub fn post_process_spawned_entities(
    mut commands: Commands,
    entity_query: Query<Entity, Added<Mesh3d>>,
) {
    for entity in entity_query.iter() {
        commands.entity(entity).insert(NoFrustumCulling);
    }
}
