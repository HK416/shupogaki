#![cfg(target_arch = "wasm32")]

use bevy::{audio::Volume, prelude::*};
use js_sys::ArrayBuffer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioContext, GainNode};

pub struct WebBgmPlugin;

impl Plugin for WebBgmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(WebBgmAudioManager::new().unwrap());
    }
}

pub struct WebBgmAudioManager {
    context: AudioContext,
    master_gain_node: GainNode,
}

impl WebBgmAudioManager {
    pub fn new() -> Result<Self, JsValue> {
        let context = AudioContext::new()?;
        let master_gain_node = context.create_gain()?;
        master_gain_node.connect_with_audio_node(&context.destination())?;
        Ok(Self {
            context,
            master_gain_node,
        })
    }

    pub fn set_volume(&self, volume: Volume) {
        self.master_gain_node.gain().set_value(volume.to_linear());
    }

    pub fn play_from_bytes(&self, source: &AudioSource, volume: Volume) {
        let array_buffer = bytes_to_array_buffer(&source.bytes);

        let context = self.context.clone();
        let gain_node = self.master_gain_node.clone();
        let volume = volume.to_linear();

        wasm_bindgen_futures::spawn_local(async move {
            let decoded_buffer = match context.decode_audio_data(&array_buffer) {
                Ok(promise) => JsFuture::from(promise).await,
                Err(e) => {
                    error!("Failed to decode audio data: {:?}", e);
                    return;
                }
            };

            let audio_buffer: AudioBuffer = match decoded_buffer {
                Ok(buffer) => buffer.dyn_into().unwrap(),
                Err(e) => {
                    error!("Error after decoding promise: {:?}", e);
                    return;
                }
            };

            if let Err(e) = play_buffer(&context, &gain_node, &audio_buffer, volume) {
                error!("Error playing buffer: {:?}", e);
            }
        });
    }
}

fn play_buffer(
    context: &AudioContext,
    gain_node: &GainNode,
    buffer: &AudioBuffer,
    volume: f32,
) -> Result<(), JsValue> {
    let source = context.create_buffer_source()?;
    source.set_buffer(Some(buffer));

    gain_node.gain().set_value(volume);
    source.set_loop(true);

    source.connect_with_audio_node(gain_node)?;
    source.start()?;
    Ok(())
}

fn bytes_to_array_buffer(bytes: &[u8]) -> ArrayBuffer {
    let uint8_array = js_sys::Uint8Array::from(bytes);
    uint8_array.buffer()
}
