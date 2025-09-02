use bevy::prelude::*;
use serde::Deserialize;

/// Represents a geometric shape that can be used for collision detection.
#[derive(Component, Clone, Copy, Deserialize)]
pub enum Collider {
    /// An Axis-Aligned Bounding Box (AABB) collider.
    Aabb { offset: Vec3, size: Vec3 },
    /// A sphere collider.
    Sphere { offset: Vec3, radius: f32 },
}

impl Collider {
    /// Checks if this collider intersects with another collider.
    /// Checks if this collider intersects with another collider, taking their world transforms into account.
    ///
    /// # Arguments
    ///
    /// * `transform` - The `Transform` of this collider.
    /// * `other` - The other collider to check against.
    /// * `other_transform` - The `Transform` of the other collider.
    ///
    /// # Returns
    ///
    /// * `true` if the colliders intersect, `false` otherwise.
    pub fn intersects(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> bool {
        match (self, other) {
            // Case 1: AABB vs AABB intersection.
            (
                Collider::Aabb {
                    offset: a_offset,
                    size: a_size,
                },
                Collider::Aabb {
                    offset: b_offset,
                    size: b_size,
                },
            ) => {
                // Calculate the minimum and maximum coordinates for both AABBs.
                let min_a = transform.translation + *a_offset - *a_size * 0.5;
                let max_a = transform.translation + *a_offset + *a_size * 0.5;
                let min_b = other_transform.translation + *b_offset - *b_size * 0.5;
                let max_b = other_transform.translation + *b_offset + *b_size * 0.5;

                // Check for overlap on each axis. If there is no gap between the boxes on any axis, they are intersecting.
                max_a.x >= min_b.x
                    && min_a.x <= max_b.x
                    && max_a.y >= min_b.y
                    && min_a.y <= max_b.y
                    && max_a.z >= min_b.z
                    && min_a.z <= max_b.z
            }
            // Case 2: AABB vs Sphere intersection.
            (
                Collider::Aabb {
                    offset: a_offset,
                    size: a_size,
                },
                Collider::Sphere {
                    offset: b_offset,
                    radius: b_radius,
                },
            ) => {
                // Find the point on the AABB closest to the sphere's center.
                let a_min = transform.translation + *a_offset - *a_size * 0.5;
                let a_max = transform.translation + *a_offset + *a_size * 0.5;
                let b_center = other_transform.translation + *b_offset;
                let closest_point = b_center.clamp(a_min, a_max);

                // Check if the squared distance from the closest point to the sphere's center
                // is less than or equal to the sphere's radius squared.
                // Using squared distances avoids a costly square root operation.
                let distance_squared = (closest_point - b_center).length_squared();
                distance_squared <= b_radius * b_radius
            }
            // Case 3: Sphere vs AABB intersection.
            (Collider::Sphere { .. }, other @ Collider::Aabb { .. }) => {
                other.intersects(other_transform, self, transform)
            }
            // Case 4: Sphere vs Sphere intersection.
            (
                Collider::Sphere {
                    offset: a_offset,
                    radius: a_radius,
                },
                Collider::Sphere {
                    offset: b_offset,
                    radius: b_radius,
                },
            ) => {
                let a_center = transform.translation + *a_offset;
                let b_center = other_transform.translation + *b_offset;

                // Calculate the squared distance between the centers of the two spheres.
                let distance_squared = (a_center - b_center).length_squared();
                let radius_sum = a_radius + b_radius;

                // Compare the squared distance with the squared sum of their radii.
                // This is more efficient than comparing the actual distances as it avoids a square root.
                distance_squared <= radius_sum * radius_sum
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_vs_aabb_intersection() {
        let aabb1 = Collider::Aabb {
            offset: Vec3::ZERO,
            size: Vec3::new(2.0, 2.0, 2.0),
        };
        let aabb2 = Collider::Aabb {
            offset: Vec3::ZERO,
            size: Vec3::new(2.0, 2.0, 2.0),
        };

        let transform1 = Transform::from_translation(Vec3::ZERO);

        // Barely touching at the edge
        let transform2 = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        assert!(aabb1.intersects(&transform1, &aabb2, &transform2));

        // Clearly overlapping
        let transform3 = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        assert!(aabb1.intersects(&transform1, &aabb2, &transform3));

        // Not intersecting
        let transform4 = Transform::from_translation(Vec3::new(2.1, 0.0, 0.0));
        assert!(!aabb1.intersects(&transform1, &aabb2, &transform4));

        // One contains another
        let aabb_small = Collider::Aabb {
            offset: Vec3::ZERO,
            size: Vec3::new(1.0, 1.0, 1.0),
        };
        let transform_center = Transform::from_translation(Vec3::new(0.5, 0.5, 0.5));
        assert!(aabb1.intersects(&transform1, &aabb_small, &transform_center));
    }

    #[test]
    fn test_sphere_vs_sphere_intersection() {
        let sphere1 = Collider::Sphere {
            offset: Vec3::ZERO,
            radius: 1.0,
        };
        let sphere2 = Collider::Sphere {
            offset: Vec3::ZERO,
            radius: 1.0,
        };

        let transform1 = Transform::from_translation(Vec3::ZERO);

        // Barely touching
        let transform2 = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        assert!(sphere1.intersects(&transform1, &sphere2, &transform2));

        // Clearly overlapping
        let transform3 = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        assert!(sphere1.intersects(&transform1, &sphere2, &transform3));

        // Not intersecting
        let transform4 = Transform::from_translation(Vec3::new(2.1, 0.0, 0.0));
        assert!(!sphere1.intersects(&transform1, &sphere2, &transform4));

        // One contains another
        let sphere_small = Collider::Sphere {
            offset: Vec3::ZERO,
            radius: 0.5,
        };
        let transform_center = Transform::from_translation(Vec3::new(0.2, 0.0, 0.0));
        assert!(sphere1.intersects(&transform1, &sphere_small, &transform_center));
    }

    #[test]
    fn test_aabb_vs_sphere_intersection() {
        let aabb = Collider::Aabb {
            offset: Vec3::ZERO,
            size: Vec3::new(2.0, 2.0, 2.0),
        };
        let sphere = Collider::Sphere {
            offset: Vec3::ZERO,
            radius: 1.0,
        };

        let aabb_transform = Transform::from_translation(Vec3::ZERO);

        // Sphere center inside AABB
        let sphere_transform_inside = Transform::from_translation(Vec3::new(0.5, 0.5, 0.5));
        assert!(aabb.intersects(&aabb_transform, &sphere, &sphere_transform_inside));

        // Barely touching at the face
        let sphere_transform_touch_face = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
        assert!(aabb.intersects(&aabb_transform, &sphere, &sphere_transform_touch_face));

        // Overlapping at the face
        let sphere_transform_overlap_face = Transform::from_translation(Vec3::new(1.5, 0.0, 0.0));
        assert!(aabb.intersects(&aabb_transform, &sphere, &sphere_transform_overlap_face));

        // Not intersecting
        let sphere_transform_no_intersect = Transform::from_translation(Vec3::new(2.1, 0.0, 0.0));
        assert!(!aabb.intersects(&aabb_transform, &sphere, &sphere_transform_no_intersect));

        // Intersecting at an edge
        let sphere_transform_edge_intersect = Transform::from_translation(Vec3::new(1.5, 1.5, 0.0));
        assert!(aabb.intersects(&aabb_transform, &sphere, &sphere_transform_edge_intersect));

        // Intersecting at a corner
        let sphere_transform_corner_intersect =
            Transform::from_translation(Vec3::new(1.5, 1.5, 1.5));
        assert!(aabb.intersects(&aabb_transform, &sphere, &sphere_transform_corner_intersect));

        // Not intersecting at a corner
        let sphere_transform_corner_no_intersect =
            Transform::from_translation(Vec3::new(2.0, 2.0, 2.0));
        assert!(!aabb.intersects(
            &aabb_transform,
            &sphere,
            &sphere_transform_corner_no_intersect
        ));
    }

    #[test]
    fn test_sphere_vs_aabb_intersection() {
        // This is just the reverse of aabb_vs_sphere, which is handled by the implementation.
        // We add a simple test for completeness.
        let aabb = Collider::Aabb {
            offset: Vec3::ZERO,
            size: Vec3::new(2.0, 2.0, 2.0),
        };
        let sphere = Collider::Sphere {
            offset: Vec3::ZERO,
            radius: 1.0,
        };

        let aabb_transform = Transform::from_translation(Vec3::ZERO);

        // Intersecting
        let sphere_transform_intersect = Transform::from_translation(Vec3::new(1.5, 0.0, 0.0));
        assert!(sphere.intersects(&sphere_transform_intersect, &aabb, &aabb_transform));

        // Not intersecting
        let sphere_transform_no_intersect = Transform::from_translation(Vec3::new(2.1, 0.0, 0.0));
        assert!(!sphere.intersects(&sphere_transform_no_intersect, &aabb, &aabb_transform));
    }
}
