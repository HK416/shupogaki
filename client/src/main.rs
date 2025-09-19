//! The main entry point for the Shupogaki game application.
//!
//! This file is responsible for setting up the Bevy application, configuring plugins,
//! defining game states, and scheduling all the systems that make up the game's logic.

mod asset;
mod collider;
mod gizmo;
mod net;
mod scene;
mod shader;
mod web;

use std::num::NonZeroU32;

// Import necessary Bevy modules.
use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    pbr::ExtendedMaterial,
    prelude::*,
};
use bevy_simple_scroll_view::ScrollViewPlugin;
use bevy_tweening::TweeningPlugin;

// Import local modules for asset handling and game scenes.
use crate::{
    asset::spawner::CustomAssetPlugin, scene::GameState,
    shader::face_mouth::FacialExpressionExtension,
};

// --- MAIN FUNCTION ---
// This is the entry point of the application.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Shupogaki 💢".into(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: false,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        desired_maximum_frame_latency: Some(NonZeroU32::new(3).unwrap()),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(LogPlugin {
                    level: if cfg!(feature = "no-debuging-log") {
                        Level::WARN
                    } else {
                        Level::INFO
                    },
                    ..Default::default()
                }),
            TweeningPlugin,
            ScrollViewPlugin,
            #[cfg(target_arch = "wasm32")]
            web::WebBgmPlugin,
        ))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, FacialExpressionExtension>,
        >::default())
        .add_plugins(CustomAssetPlugin)
        .add_plugins(gizmo::GizmoPlugin)
        .add_plugins(scene::StatePlugin)
        .init_state::<GameState>()
        .run();
}
