use bevy::prelude::*;
use serde::Deserialize;

/// Represents a geometric shape that can be used for collision detection.
#[derive(Component, Deserialize)]
pub enum Collider {
    /// An Axis-Aligned Bounding Box (AABB) collider.
    Aabb { center: Vec3, size: Vec3 },
    /// A sphere collider.
    Sphere { center: Vec3, radius: f32 },
}

impl Collider {
    /// Checks if this collider intersects with another collider.
    pub fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            // Case 1: AABB vs AABB intersection.
            (
                Collider::Aabb {
                    center: a_center,
                    size: a_size,
                },
                Collider::Aabb {
                    center: b_center,
                    size: b_size,
                },
            ) => {
                // Calculate the minimum and maximum coordinates for both AABBs.
                let min_a = *a_center - *a_size * 0.5;
                let max_a = *a_center + *a_size * 0.5;
                let min_b = *b_center - *b_size * 0.5;
                let max_b = *b_center + *b_size * 0.5;

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
                    center: a_center,
                    size: a_size,
                },
                Collider::Sphere {
                    center: b_center,
                    radius: b_radius,
                },
            ) => {
                // Find the point on the AABB closest to the sphere's center.
                let a_min = *a_center - *a_size * 0.5;
                let a_max = *a_center + *a_size * 0.5;
                let closest_point = b_center.clamp(a_min, a_max);

                // Check if the squared distance from the closest point to the sphere's center
                // is less than or equal to the sphere's radius squared.
                // Using squared distances avoids a costly square root operation.
                let distance_squared = (closest_point - *b_center).length_squared();
                distance_squared <= *b_radius * *b_radius
            }
            // Case 3: Sphere vs AABB intersection.
            (Collider::Sphere { .. }, other @ Collider::Aabb { .. }) => {
                // Delegate to the AABB vs Sphere implementation to avoid code duplication.
                // The logic is symmetrical.
                other.intersects(self)
            }
            // Case 4: Sphere vs Sphere intersection.
            (
                Collider::Sphere {
                    center: a_center,
                    radius: a_radius,
                },
                Collider::Sphere {
                    center: b_center,
                    radius: b_radius,
                },
            ) => {
                // Calculate the squared distance between the centers of the two spheres.
                let distance_squared = (*a_center - *b_center).length_squared();
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

    // AABB vs AABB
    #[test]
    fn test_aabb_aabb_intersect() {
        let a = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        let b = Collider::Aabb {
            center: Vec3::new(1.5, 0.0, 0.0),
            size: Vec3::ONE,
        };
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn test_aabb_aabb_no_intersect() {
        let a = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        let b = Collider::Aabb {
            center: Vec3::new(2.1, 0.0, 0.0),
            size: Vec3::ONE,
        };
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    // Sphere vs Sphere
    #[test]
    fn test_sphere_sphere_intersect() {
        let a = Collider::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
        };
        let b = Collider::Sphere {
            center: Vec3::new(1.5, 0.0, 0.0),
            radius: 1.0,
        };
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn test_sphere_sphere_no_intersect() {
        let a = Collider::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
        };
        let b = Collider::Sphere {
            center: Vec3::new(2.1, 0.0, 0.0),
            radius: 1.0,
        };
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    // AABB vs Sphere
    #[test]
    fn test_aabb_sphere_intersect() {
        let aabb = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        let sphere = Collider::Sphere {
            center: Vec3::new(1.5, 0.0, 0.0),
            radius: 1.0,
        };
        assert!(aabb.intersects(&sphere));
        assert!(sphere.intersects(&aabb));
    }

    #[test]
    fn test_aabb_sphere_no_intersect() {
        let aabb = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        let sphere = Collider::Sphere {
            center: Vec3::new(2.1, 0.0, 0.0),
            radius: 1.0,
        };
        assert!(!aabb.intersects(&sphere));
        assert!(!sphere.intersects(&aabb));
    }

    #[test]
    fn test_aabb_sphere_intersect_corner() {
        let aabb = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        // Sphere center is outside the AABB, but close enough to a corner to intersect
        let sphere = Collider::Sphere {
            center: Vec3::new(1.5, 1.5, 0.0),
            radius: 1.0,
        };
        // Closest point on AABB is (1.0, 1.0, 0.0)
        // Distance to sphere center is sqrt((1.5-1.0)^2 + (1.5-1.0)^2) = sqrt(0.25 + 0.25) = sqrt(0.5) ~= 0.707
        // Since 0.707 < 1.0 (radius), they intersect.
        assert!(aabb.intersects(&sphere));
        assert!(sphere.intersects(&aabb));
    }

    #[test]
    fn test_aabb_sphere_no_intersect_corner() {
        let aabb = Collider::Aabb {
            center: Vec3::ZERO,
            size: Vec3::ONE,
        };
        let sphere = Collider::Sphere {
            center: Vec3::new(1.8, 1.8, 0.0),
            radius: 1.0,
        };
        // Closest point on AABB is (1.0, 1.0, 0.0)
        // Distance to sphere center is sqrt((1.8-1.0)^2 + (1.8-1.0)^2) = sqrt(0.64 + 0.64) = sqrt(1.28) ~= 1.13
        // Since 1.13 > 1.0 (radius), they do not intersect.
        assert!(!sphere.intersects(&aabb));
        assert!(!aabb.intersects(&sphere));
    }
}
