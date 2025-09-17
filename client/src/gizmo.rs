use bevy::prelude::*;

// Conditionally import the Collider for debug gizmos.
#[cfg(not(feature = "no-debuging-gizmo"))]
use crate::collider::Collider;

// --- PLUGIN ---

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "no-debuging-gizmo"))]
        app.add_systems(PostUpdate, (update_gizmo_config, draw_collider_gizmos));
    }
}

// --- DEBUG GIZMO SYSTEMS ---
// These systems are only compiled if the "no-debuging-gizmo" feature is NOT enabled.

/// Toggles the visibility of debug gizmos when the F4 key is pressed.
// This system is only compiled if the "no-debuging-gizmo" feature is NOT enabled.
#[cfg(not(feature = "no-debuging-gizmo"))]
pub fn update_gizmo_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Check if F4 was just pressed.
    if keyboard_input.just_pressed(KeyCode::F4) {
        // Iterate through all gizmo configurations and toggle their `enabled` flag.
        for (_, config, _) in config_store.iter_mut() {
            // `^=` is the XOR assignment operator, a concise way to toggle a boolean.
            config.enabled ^= true;
        }
    }
}

/// Draws visual representations (gizmos) for all `Collider` components in the scene.
// This system is only compiled if the "no-debuging-gizmo" feature is NOT enabled.
#[cfg(not(feature = "no-debuging-gizmo"))]
pub fn draw_collider_gizmos(mut gizmos: Gizmos, query: Query<(&Collider, &Transform)>) {
    const GIZMO_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);

    // Iterate over all entities with a Collider component.
    for (collider, transform) in query.iter() {
        // Draw axes to show the orientation of the entity.
        gizmos.axes(*transform, 2.0);

        match collider {
            Collider::Aabb { offset, size } => {
                let center = transform.translation + *offset;
                gizmos.cuboid(
                    Transform::from_translation(center).with_scale(*size),
                    GIZMO_COLOR,
                );
            }
            Collider::Sphere { offset, radius } => {
                let center = transform.translation + *offset;
                gizmos.sphere(Isometry3d::from_translation(center), *radius, GIZMO_COLOR);
            }
        }
    }
}
