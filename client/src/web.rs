#![cfg(target_arch = "wasm32")]

use bevy::{
    audio::{AudioSource, PlaybackMode, Volume},
    platform::collections::HashMap,
    prelude::*,
};
use flume::{Receiver, Sender};
use js_sys::ArrayBuffer;
use wasm_bindgen::prelude::*;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};

pub struct WebAudioPlugin;

impl Plugin for WebAudioPlugin {
    fn build(&self, app: &mut App) {
        let context = WebAudioContext::new().expect("Web browser does not support AudioContext");
        app.insert_non_send_resource(context)
            .init_non_send_resource::<WebAudioSources>()
            .init_non_send_resource::<WebAudioBufferCache>()
            .init_non_send_resource::<WebAudioDecodedChannel>()
            .add_systems(
                Update,
                (
                    system_spawn_new_web_players.in_set(WebAudioSet::Spawn),
                    system_setup_decoded_audio
                        .in_set(WebAudioSet::Setup)
                        .after(WebAudioSet::Spawn),
                    system_sync_playback_state
                        .in_set(WebAudioSet::Sync)
                        .after(WebAudioSet::Setup),
                    (
                        system_cleanup_finished_sounds,
                        system_despawn_finished_sounds,
                    )
                        .in_set(WebAudioSet::Cleanup)
                        .after(WebAudioSet::Sync),
                ),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum WebAudioSet {
    Spawn,
    Setup,
    Sync,
    Cleanup,
}

// --- COMPONENTS ---

#[derive(Component)]
pub struct WebAudioPlayer(Handle<AudioSource>);

impl WebAudioPlayer {
    pub fn new(source: Handle<AudioSource>) -> Self {
        Self(source)
    }
}

#[derive(Component, Clone, Copy)]
pub struct WebPlaybackSettings {
    pub mode: PlaybackMode,
    pub volume: Volume,
    pub paused: bool,
}

impl Default for WebPlaybackSettings {
    fn default() -> Self {
        Self {
            mode: PlaybackMode::Despawn,
            volume: Volume::default(),
            paused: false,
        }
    }
}

#[allow(dead_code)]
impl WebPlaybackSettings {
    pub const ONCE: WebPlaybackSettings = WebPlaybackSettings {
        mode: PlaybackMode::Once,
        volume: Volume::Linear(1.0),
        paused: false,
    };

    pub const LOOP: WebPlaybackSettings = WebPlaybackSettings {
        mode: PlaybackMode::Loop,
        volume: Volume::Linear(1.0),
        paused: false,
    };

    pub const DESPAWN: WebPlaybackSettings = WebPlaybackSettings {
        mode: PlaybackMode::Despawn,
        volume: Volume::Linear(1.0),
        paused: false,
    };

    pub const REMOVE: WebPlaybackSettings = WebPlaybackSettings {
        mode: PlaybackMode::Remove,
        volume: Volume::Linear(1.0),
        paused: false,
    };

    pub const fn with_volume(mut self, volume: Volume) -> Self {
        self.volume = volume;
        self
    }
}

pub enum PlaybackState {
    Playing { start_time: f64 },
    Paused { elapsed_before_pause: f64 },
    Stopped,
}

#[derive(Component)]
pub struct PlaybackTracker {
    state: PlaybackState,
    duration_secs: f64,
}

#[derive(Component)]
pub struct WebPlaybackDespawnMarker;

// --- RESOURCES ---

struct WebAudioContext(AudioContext);

impl WebAudioContext {
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self(AudioContext::new()?))
    }
}

struct WebAudioEntry {
    gain_node: GainNode,
    source_node: AudioBufferSourceNode,
}

struct WebAudioDecodedChannel {
    sender: Sender<(Entity, AudioBuffer, AssetId<AudioSource>)>,
    receiver: Receiver<(Entity, AudioBuffer, AssetId<AudioSource>)>,
}

impl Default for WebAudioDecodedChannel {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }
}

#[derive(Default)]
struct WebAudioSources(HashMap<Entity, WebAudioEntry>);

#[derive(Default)]
struct WebAudioBufferCache(HashMap<AssetId<AudioSource>, AudioBuffer>);

// --- UPDATE SYSTEMS ---

fn system_spawn_new_web_players(
    mut commands: Commands,
    query: Query<(Entity, &WebAudioPlayer, Option<&WebPlaybackSettings>), Added<WebAudioPlayer>>,
    audio_assets: Res<Assets<AudioSource>>,
    context: NonSend<WebAudioContext>,
    cache: NonSend<WebAudioBufferCache>,
    channel: NonSend<WebAudioDecodedChannel>,
) {
    for (entity, player, settings) in query.iter() {
        let Some(source) = audio_assets.get(&player.0) else {
            continue;
        };

        let asset_id = player.0.id();
        let sender = channel.sender.clone();
        if let Some(cached_buffer) = cache.0.get(&asset_id) {
            info!("Cache hit for audio asset: {:?}", &asset_id);
            sender
                .send((entity, cached_buffer.clone(), asset_id))
                .unwrap();
        } else {
            let array_buffer = bytes_to_array_buffer(&source.bytes);
            let promise = context.0.decode_audio_data(&array_buffer).unwrap();
            let future = wasm_bindgen_futures::JsFuture::from(promise);

            wasm_bindgen_futures::spawn_local(async move {
                match future.await {
                    Ok(decoded_buffer) => {
                        let audio_buffer: AudioBuffer = decoded_buffer.dyn_into().unwrap();
                        sender.send((entity, audio_buffer, asset_id)).unwrap();
                    }
                    Err(e) => {
                        error!("Error decoding audio data: {:?}", e);
                    }
                }
            });
        }

        if settings.is_none() {
            commands
                .entity(entity)
                .insert(WebPlaybackSettings::default());
        }
    }
}

fn bytes_to_array_buffer(bytes: &[u8]) -> ArrayBuffer {
    let uint8_array = js_sys::Uint8Array::from(bytes);
    uint8_array.buffer()
}

fn system_setup_decoded_audio(
    mut commands: Commands,
    mut sources: NonSendMut<WebAudioSources>,
    context: NonSend<WebAudioContext>,
    mut cache: NonSendMut<WebAudioBufferCache>,
    channel: NonSend<WebAudioDecodedChannel>,
    query: Query<&WebPlaybackSettings>,
) {
    while let Ok((entity, audio_buffer, asset_id)) = channel.receiver.try_recv() {
        cache.0.insert(asset_id, audio_buffer.clone());

        if let Ok(settings) = query.get(entity) {
            let gain_node = context.0.create_gain().unwrap();
            gain_node
                .connect_with_audio_node(&context.0.destination())
                .unwrap();
            gain_node.gain().set_value(settings.volume.to_linear());

            let source_node = context.0.create_buffer_source().unwrap();
            source_node.connect_with_audio_node(&gain_node).unwrap();
            source_node.set_buffer(Some(&audio_buffer));

            match settings.mode {
                PlaybackMode::Loop => {
                    source_node.set_loop(true);
                }
                PlaybackMode::Despawn => {
                    commands.entity(entity).insert(WebPlaybackDespawnMarker);
                }
                _ => { /* empty */ }
            };

            let mut tracker = PlaybackTracker {
                state: PlaybackState::Stopped,
                duration_secs: audio_buffer.duration(),
            };

            if !settings.paused {
                source_node.start().unwrap();
                tracker.state = PlaybackState::Playing {
                    start_time: context.0.current_time(),
                };
                info!("Play Sound: {}", asset_id);
            };

            commands.entity(entity).insert(tracker);
            sources.0.insert(
                entity,
                WebAudioEntry {
                    gain_node,
                    source_node,
                },
            );
        }
    }
}

fn system_sync_playback_state(
    context: NonSend<WebAudioContext>,
    cache: NonSend<WebAudioBufferCache>,
    mut sources: NonSendMut<WebAudioSources>,
    mut query: Query<
        (
            Entity,
            &WebAudioPlayer,
            &WebPlaybackSettings,
            &mut PlaybackTracker,
        ),
        Changed<WebPlaybackSettings>,
    >,
) {
    for (entity, player, settings, mut tracker) in query.iter_mut() {
        if let Some(mut entry) = sources.0.remove(&entity) {
            entry
                .gain_node
                .gain()
                .set_value(settings.volume.to_linear());

            let current_time = context.0.current_time();
            let is_playing = matches!(tracker.state, PlaybackState::Playing { .. });

            if settings.paused
                && is_playing
                && let PlaybackState::Playing { start_time } = tracker.state
            {
                entry.source_node.stop().ok();
                tracker.state = PlaybackState::Paused {
                    elapsed_before_pause: current_time - start_time,
                };
            } else if !settings.paused
                && !is_playing
                && let PlaybackState::Paused {
                    elapsed_before_pause,
                } = tracker.state
            {
                let asset_id = player.0.id();
                let audio_buffer = cache.0.get(&asset_id).unwrap();

                let new_source = context.0.create_buffer_source().unwrap();
                new_source
                    .connect_with_audio_node(&entry.gain_node)
                    .unwrap();
                new_source.set_buffer(Some(audio_buffer));
                if matches!(settings.mode, PlaybackMode::Loop) {
                    new_source.set_loop(true);
                }

                new_source
                    .start_with_when_and_grain_offset(0.0, elapsed_before_pause)
                    .unwrap();
                entry.source_node = new_source;

                tracker.state = PlaybackState::Playing {
                    start_time: current_time - elapsed_before_pause,
                };
            }

            sources.0.insert(entity, entry);
        }
    }
}

fn system_cleanup_finished_sounds(
    mut removed: RemovedComponents<WebAudioPlayer>,
    mut sources: NonSendMut<WebAudioSources>,
) {
    for entity in removed.read() {
        if let Some(entry) = sources.0.remove(&entity) {
            entry.source_node.stop().ok();
            entry.source_node.disconnect().ok();
            entry.gain_node.disconnect().ok();
        }
    }
}

fn system_despawn_finished_sounds(
    mut commands: Commands,
    context: NonSend<WebAudioContext>,
    query: Query<(Entity, &PlaybackTracker), With<WebPlaybackDespawnMarker>>,
) {
    for (entity, tracker) in query.iter() {
        if let PlaybackState::Playing { start_time } = tracker.state {
            let current_time = context.0.current_time();
            let elapsed_time = current_time - start_time;

            if elapsed_time >= tracker.duration_secs {
                commands.entity(entity).despawn();
                info!("Despawning finished sound for entity: {:?}", entity);
            }
        }
    }
}
