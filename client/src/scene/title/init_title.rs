// Import necessary Bevy modules.
use bevy::{audio::Volume, prelude::*, render::view::NoFrustumCulling};

use crate::asset::{
    animation::AnimationClipHandle,
    sound::SystemVolume,
    spawner::{SpawnModel, TranslatableText},
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitTitle),
            (debug_label, play_loading_sound, spawn_entities),
        )
        .add_systems(OnExit(GameState::InitTitle), remove_resource)
        .add_systems(
            Update,
            (
                disable_frustum_culling,
                handle_spawn_request,
                check_loading_progress,
                update_loading_progress,
            )
                .run_if(in_state(GameState::InitTitle)),
        );
    }
}

// --- SETUP SYSTEMS ---

fn debug_label() {
    info!("Current State: InitTitle");
}

fn play_loading_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    system_volume: Res<SystemVolume>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(SOUND_PATH_UI_LOADING)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(system_volume.effect_percentage())),
        EffectSound,
    ));
}

fn spawn_entities(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_entities = LoadingEntities::default();
    spawn_title_entities(&mut commands, &asset_server, &mut loading_entities);
    spawn_title_ui_entities(&mut commands, &asset_server, &mut loading_entities);

    // --- Resource Insertion ---
    commands.insert_resource(loading_entities);
}

fn spawn_title_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    let model = asset_server.load(MODEL_PATH_PLANE_0);
    for i in 0..3 {
        let entity = commands
            .spawn((
                SpawnModel(model.clone()),
                Transform::from_xyz(0.0, 0.0, -30.0 * (i + 1) as f32),
                Visibility::Hidden,
                SpawnRequest,
            ))
            .id();
        loading_entities.handles.push(entity);
    }

    let model = asset_server.load(MODEL_PATH_PLANE_999);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            SpawnRequest,
        ))
        .id();
    loading_entities.handles.push(entity);

    let entity = commands
        .spawn((
            Transform::from_translation(TRAIN_POSITION)
                .with_rotation(Quat::from_rotation_y(180f32.to_radians())),
            Visibility::Hidden,
            SpawnRequest,
        ))
        .with_children(|parent| {
            let model = asset_server.load(MODEL_PATH_TOY_TRAIN_00);
            parent.spawn((
                SpawnModel(model),
                Transform::from_xyz(0.0, 0.0, -1.275),
                Visibility::Inherited,
            ));

            let model = asset_server.load(MODEL_PATH_TOY_TRAIN_01);
            parent.spawn((
                SpawnModel(model),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::Inherited,
            ));

            let model = asset_server.load(MODEL_PATH_TOY_TRAIN_02);
            parent.spawn((
                SpawnModel(model),
                Transform::from_xyz(0.0, 0.0, 1.08),
                Visibility::Inherited,
            ));
        })
        .id();
    loading_entities.handles.push(entity);

    let clip = asset_server.load(ANIM_PATH_HIKARI_CAFE_IDLE);
    let model = asset_server.load(MODEL_PATH_HIKARI);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::from_translation(title::HIKARI_POSITION)
                .looking_at(title::CAMERA_POSITION.with_y(0.0), Vec3::Y),
            AnimationClipHandle(clip),
            Visibility::Hidden,
            SpawnRequest,
        ))
        .id();
    loading_entities.handles.push(entity);

    let clip = asset_server.load(ANIM_PATH_NOZOMI_CAFE_IDLE);
    let model = asset_server.load(MODEL_PATH_NOZOMI);
    let entity = commands
        .spawn((
            SpawnModel(model),
            Transform::from_translation(title::NOZOMI_POSITION)
                .looking_at(title::CAMERA_POSITION.with_y(0.0), Vec3::Y),
            AnimationClipHandle(clip),
            Visibility::Hidden,
            SpawnRequest,
        ))
        .id();
    loading_entities.handles.push(entity);
}

fn spawn_title_ui_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    loading_entities: &mut LoadingEntities,
) {
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Vw(5.0),
                bottom: Val::Vh(10.0),
                width: Val::Vw(30.0),
                height: Val::Vh(50.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            SpawnRequest,
        ))
        .with_children(|parent| {
            let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
            parent.spawn((
                Text::new("Start Game"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextShadow::default(),
                TranslatableText("start".to_string()),
                ResizableFont::vertical(1280.0, 102.0),
                Node::default(),
                Visibility::Hidden,
                UI::StartButton,
                Button,
            ));

            parent.spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(5.0),
                ..Default::default()
            },));

            parent.spawn((
                Text::new("Settings"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextShadow::default(),
                TranslatableText("option".to_string()),
                ResizableFont::vertical(1280.0, 102.0),
                Node::default(),
                Visibility::Hidden,
                UI::OptionButton,
                Button,
            ));

            parent.spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(5.0),
                ..Default::default()
            },));

            parent.spawn((
                Text::new("Ranking"),
                TextFont::from_font(font.clone()),
                TextLayout::new_with_justify(JustifyText::Center),
                TextShadow::default(),
                TranslatableText("ranking".to_string()),
                ResizableFont::vertical(1280.0, 102.0),
                Node::default(),
                Visibility::Hidden,
                UI::RankButton,
                Button,
            ));
        })
        .id();
    loading_entities.handles.push(entity);
}

// --- CLEANUP SYSTEMS ---

fn remove_resource(mut commands: Commands) {
    commands.remove_resource::<LoadingEntities>();
}

// --- UPDATE SYSTEMS ---

fn disable_frustum_culling(mut commands: Commands, query: Query<Entity, Added<Mesh3d>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(NoFrustumCulling);
    }
}

fn handle_spawn_request(mut commands: Commands, query: Query<Entity, Added<SpawnRequest>>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<SpawnRequest>()
            .insert(TitleStateRoot);
    }
}

fn check_loading_progress(
    mut next_state: ResMut<NextState<GameState>>,
    spawn_request_entities: Query<(), With<SpawnRequest>>,
    model_spawn_entities: Query<(), With<SpawnModel>>,
) {
    if spawn_request_entities.is_empty() && model_spawn_entities.is_empty() {
        next_state.set(GameState::InitRank);
    }
}

fn update_loading_progress(
    loading_entitis: Res<LoadingEntities>,
    spawn_request_entities: Query<(), With<SpawnRequest>>,
    mut loading_bar_entities: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = loading_bar_entities.single_mut() {
        let loaded_count = loading_entitis
            .handles
            .iter()
            .filter(|&&entity| !spawn_request_entities.contains(entity))
            .count();

        let total_count = loading_entitis.handles.len();
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0
        };

        node.width = Val::Percent(progress * 100.0);
    }
}
