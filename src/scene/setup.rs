// Import necessary Bevy modules.
use bevy::prelude::*;

use crate::asset::{
    locale::{CurrentLocale, Locale, LocalizationAssets, LocalizationData},
    sound::SystemVolume,
};

use super::*;

// --- PLUGIN ---

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register systems to run when entering the `GameState::Setup` state.
            .add_systems(
                OnEnter(GameState::Setup),
                (
                    debug_label,
                    setup_locale,
                    setup_system_volume,
                    load_necessary_assets,
                    setup_loading_screen,
                ),
            )
            // Register systems to run every frame while in the `GameState::Setup` state.
            .add_systems(
                Update,
                (check_loading_progress, update_loading_bar).run_if(in_state(GameState::Setup)),
            );
    }
}

// --- SETUP SYSTEMS ---

/// Prints a debug message to the console indicating the current game state.
fn debug_label() {
    info!("Current State: Setup");
}

/// Sets up the initial locale for the game.
/// For web builds (wasm32), it detects the browser's language.
/// Otherwise, it falls back to the default language.
fn setup_locale(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;

        // Attempt to get the browser's language setting.
        let locale = window()
            .and_then(|w| w.navigator().language())
            .unwrap_or_else(|| "en-US".to_string());
        info!("Detected browser language: {}", locale);

        // Match the browser language to a supported `Locale`.
        let locale = match locale.as_str() {
            "en-US" => Locale::En,
            "ja-JP" => Locale::Ja,
            "ko-KR" => Locale::Ko,
            _ => Locale::En, // Default to English if the language is not supported.
        };

        commands.insert_resource(CurrentLocale(locale));
        info!("Use language: {}", locale);
        return;
    }

    // For non-web builds, use the default language.
    #[allow(unreachable_code)]
    commands.insert_resource(CurrentLocale::default());
    info!("Use default language: {}", Locale::default());
}

/// Initializes and inserts the default system volume as a resource.
fn setup_system_volume(mut commands: Commands) {
    commands.insert_resource(SystemVolume::default());
}

/// Begins loading essential assets required for the game to start,
/// such as localization files and fonts. These assets are tracked for the loading screen.
fn load_necessary_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loading_assets = SystemAssets::default();
    let mut localizations = LocalizationAssets::default();

    // --- Locale Loading ---
    // Load localization data for each supported language.
    let handle: Handle<LocalizationData> = asset_server.load(LOCALE_PATH_EN);
    localizations.locale.insert(Locale::En, handle.clone());
    loading_assets.handles.push(handle.into());

    let handle: Handle<LocalizationData> = asset_server.load(LOCALE_PATH_JA);
    localizations.locale.insert(Locale::Ja, handle.clone());
    loading_assets.handles.push(handle.into());

    let handle: Handle<LocalizationData> = asset_server.load(LOCALE_PATH_KO);
    localizations.locale.insert(Locale::Ko, handle.clone());
    loading_assets.handles.push(handle.into());

    // --- Font Loading ---
    // Load the primary font used in the UI.
    let font: Handle<Font> = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
    loading_assets.handles.push(font.into());

    // --- Resource Insertion ---
    // Insert the asset collections as resources for other systems to use.
    commands.insert_resource(loading_assets);
    commands.insert_resource(localizations);
}

/// Spawns the UI entities for the loading screen, including the camera,
/// "Now Loading..." text, and the progress bar.
fn setup_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera for the loading screen UI.
    commands.spawn((Camera2d, LoadingStateRoot));

    // Create the main UI container for the loading elements.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(20.0),
                height: Val::Vh(5.0),
                bottom: Val::Vh(3.0),
                right: Val::Vw(3.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            LoadingStateRoot,
        ))
        .with_children(|parent| {
            // Container for the loading text.
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(88.0),
                    ..Default::default()
                },))
                .with_children(|parent| {
                    // Spawn the "Now Loading..." text element.
                    let font = asset_server.load(FONT_PATH_NOTOSANS_BOLD);
                    parent.spawn((
                        Text::new("Now Loading..."),
                        TextFont::from_font(font).with_font_size(24.0),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextColor::WHITE,
                        ResizableFont::vertical(1280.0, 24.0),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            overflow: Overflow::hidden(),
                            ..Default::default()
                        },
                        LoadingText,
                        ZIndex(2),
                    ));
                });

            // Container for the loading progress bar.
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(12.0),
                        border: UiRect::all(Val::Percent(0.25)),
                        ..Default::default()
                    },
                    BorderColor(Color::WHITE),
                    BorderRadius::all(Val::Percent(50.0)),
                ))
                .with_children(|parent| {
                    // The actual loading bar that will be filled.
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0), // Starts at 0% width.
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BorderRadius::all(Val::Percent(50.0)),
                        BackgroundColor(LOADING_BAR_COLOR),
                        LoadingBar, // Tag component for querying.
                        ZIndex(1),
                    ));
                });
        });

    // Set the background color to black during setup.
    commands.insert_resource(ClearColor(Color::BLACK));
}

// --- UPDATE SYSTEMS ---

/// Checks the loading status of all assets tracked in `SystemAssets`.
/// If all assets are loaded, it transitions the game to the `GameState::Initialize` state.
fn check_loading_progress(
    asset_server: Res<AssetServer>,
    loading_assets: ResMut<SystemAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check if all handles in the loading list have finished loading, including their dependencies.
    let all_loaded = loading_assets
        .handles
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    if all_loaded {
        // Transition to the next state once loading is complete.
        next_state.set(GameState::Initialize);
    }
}

/// Updates the width of the loading bar UI element based on the current asset loading progress.
fn update_loading_bar(
    asset_server: Res<AssetServer>,
    loading_assets: Res<SystemAssets>,
    mut query: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = query.single_mut() {
        // Count how many assets have successfully loaded.
        let loaded_count = loading_assets
            .handles
            .iter()
            .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
            .count();

        let total_count = loading_assets.handles.len();
        // Calculate the progress percentage.
        let progress = if total_count > 0 {
            loaded_count as f32 / total_count as f32
        } else {
            1.0 // Avoid division by zero if there are no assets to load.
        };

        // Update the width of the loading bar node.
        node.width = Val::Percent(progress * 100.0);
    }
}
