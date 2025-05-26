use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use slab::Slab;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Default)]
pub struct Audio {
    stream_handle: Option<OutputStreamHandle>,
    audio_sources: HashMap<String, AudioSource>,
    sinks: Slab<Sink>,
}

impl Audio {
    pub fn resume_audio_context(&mut self) {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            log::info!("Default audio device found.");
            // We leak `OutputStream` to prevent the audio from stopping.
            core::mem::forget(stream);
            self.stream_handle = Some(stream_handle);
        } else {
            log::warn!("No audio device found.");
        };
    }
    pub fn load_source(&mut self, key: &str, source_bytes: Vec<u8>) {
        let source = AudioSource::new(source_bytes);
        self.audio_sources.insert(key.to_string(), source);
    }
    pub fn play_sound(&mut self, source_key: &str) -> Option<usize> {
        self.play_sound_with_volume(source_key, 1.0)
    }
    pub fn play_sound_with_volume(&mut self, source_key: &str, volume: f32) -> Option<usize> {
        let Some(stream_handle) = self.stream_handle.as_ref() else {
            log::warn!("Audio output unavailable, cannot play sound");
            return None;
        };
        let Some(source) = self.audio_sources.get(source_key) else {
            log::warn!("Unavailable audio source({source_key})");
            return None;
        };
        let sink = Sink::try_new(&stream_handle).expect("Could not create audio sink");
        sink.append(source.decoder());
        sink.set_volume(volume);
        sink.play();
        Some(self.sinks.insert(sink))
    }
    pub fn get_sink(&self, key: usize) -> Option<&Sink> {
        self.sinks.get(key)
    }
    pub fn clean_finished_sink(&mut self) {
        // self.sinks
        //     .retain(|_key, sink| if sink.empty() { false } else { true });
    }
}

pub struct AudioSource {
    pub bytes: Arc<[u8]>,
}

impl AudioSource {
    fn new(bytes: Vec<u8>) -> AudioSource {
        Self {
            bytes: bytes.into(),
        }
    }
    pub fn decoder(&self) -> Decoder<Cursor<Arc<[u8]>>> {
        Decoder::new(Cursor::new(self.bytes.clone())).unwrap()
    }
}
