use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::ScalingMode};

// 플레이어 엔티티를 식별하기 위한 컴포넌트
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shupogaki 💢".into(),
                resolution: (1280.0, 720.0).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.3, 0.5, 0.3).into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Cube Player
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.8, 0.7, 0.6).into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.5, 8.0),
        Player,
    ));

    // Directional Light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
    ));

    // 정사영 투영을 사용하는 3D 카메라
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scale: 1.25,
            scaling_mode: ScalingMode::Fixed {
                width: 16.0,
                height: 9.0,
            },
            near: 0.1,
            far: 100.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(12.0, 9.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
