// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*};

#[cfg(target_arch = "wasm32")]
use crate::web::{WebAudioPlayer, WebPlaybackSettings, start_game_tutorial};

use crate::{
    asset::{
        animation::AnimationClipHandle, locale::CurrentLocale, material::EyeMouthMaterial,
        sound::SystemVolume,
    },
    shader::face_mouth::EyeMouth,
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Title),
            (
                debug_label,
                show_entities,
                show_interfaces,
                spawn_camera_and_light,
                play_animation,
                setup_background_sound,
                setup_mouth_expression,
            ),
        )
        .add_systems(OnExit(GameState::Title), hide_interfaces)
        .add_systems(
            PreUpdate,
            title_button_systems.run_if(in_state(GameState::Title)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: Title");
}

fn show_entities(mut query: Query<&mut Visibility, (With<TitleStateRoot>, Without<UI>)>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

fn show_interfaces(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::HighScore | UI::StartButton | UI::OptionButton | UI::TutorialButton => {
                *visibility = Visibility::Visible
            }
            _ => { /* empty */ }
        }
    }
}

fn spawn_camera_and_light(
    mut commands: Commands,
    light_query: Query<(), With<DirectionalLight>>,
    camera_query: Query<(), With<Camera3d>>,
) {
    if light_query.single().is_err() {
        commands.spawn((
            DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            Transform::from_xyz(8.0, 12.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            TitleStateRoot,
        ));
    }

    if camera_query.single().is_err() {
        commands.spawn((
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                fov: 50f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
                near: 0.1,
                far: 100.0,
            }),
            Transform::from_translation(CAMERA_POSITION).looking_to(CAMERA_DIRECTION, Vec3::Y),
            TitleStateRoot,
        ));
    }

    commands.insert_resource(ClearColor(CLEAR_COLOR));
}

fn play_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    query: Query<(Entity, &AnimationClipHandle)>,
) {
    for (entity, clip) in query.iter() {
        let (graph, animation_index) = AnimationGraph::from_clip(clip.0.clone());
        let mut player = AnimationPlayer::default();
        player.play(animation_index).repeat();

        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graphs.add(graph)), player))
            .remove::<AnimationClipHandle>();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_background_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    query: Query<(), With<BackgroundSound>>,
) {
    if query.single().is_err() {
        commands.spawn((
            AudioPlayer::new(asset_server.load(SOUND_PATH_BACKGROUND)),
            PlaybackSettings::LOOP
                .with_volume(Volume::Linear(system_volume.background_percentage())),
            BackgroundSound,
        ));
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_background_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    query: Query<(), With<BackgroundSound>>,
) {
    if query.single().is_err() {
        commands.spawn((
            WebAudioPlayer::new(asset_server.load(SOUND_PATH_BACKGROUND)),
            WebPlaybackSettings::LOOP
                .with_volume(Volume::Linear(system_volume.background_percentage())),
            BackgroundSound,
        ));
    }
}

fn setup_mouth_expression(
    mut materials: ResMut<Assets<EyeMouthMaterial>>,
    query: Query<&EyeMouth>,
) {
    for mouth in query.iter() {
        if let Some(material) = materials.get_mut(&mouth.0) {
            material.extension.uniform.index.x = 0;
        }
    }
}

// --- CLEANUP SYSTEM ---

fn hide_interfaces(mut query: Query<(&UI, &mut Visibility)>) {
    for (&ui, mut visibility) in query.iter_mut() {
        match ui {
            UI::HighScore | UI::StartButton | UI::OptionButton | UI::TutorialButton => {
                *visibility = Visibility::Hidden
            }
            _ => { /* empty */ }
        }
    }
}

// --- UPDATE SYSTEM ---

#[allow(clippy::type_complexity)]
fn title_button_systems(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
    #[allow(unused_variables)] current_locale: Res<CurrentLocale>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&UI, &Interaction, &mut TextColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (&ui, interaction, mut text_color) in interaction_query.iter_mut() {
        match (ui, interaction) {
            (UI::StartButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::StartButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                next_state.set(GameState::Title2InGame);
            }
            (UI::StartButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            (UI::OptionButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::OptionButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                next_state.set(GameState::Option);
            }
            (UI::OptionButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            (UI::TutorialButton, Interaction::Hovered) => {
                *text_color = TextColor(Color::WHITE.darker(0.3));
                play_button_sound_when_hovered(&mut commands, &asset_server, &system_volume);
            }
            (UI::TutorialButton, Interaction::Pressed) => {
                *text_color = TextColor(Color::WHITE.darker(0.5));
                play_button_sound_when_pressed(&mut commands, &asset_server, &system_volume);
                #[cfg(target_arch = "wasm32")]
                start_game_tutorial(&current_locale.0.to_string());
            }
            (UI::TutorialButton, Interaction::None) => {
                *text_color = TextColor(Color::WHITE);
            }
            _ => { /* empty */ }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_button_sound_when_hovered(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_button_sound_when_hovered(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn play_button_sound_when_pressed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_TOUCH)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

#[cfg(target_arch = "wasm32")]
fn play_button_sound_when_pressed(
    commands: &mut Commands,
    asset_server: &AssetServer,
    system_volume: &SystemVolume,
) {
    commands.spawn((
        WebAudioPlayer::new(asset_server.load(SOUND_PATH_UI_BUTTON_TOUCH)),
        WebPlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}
