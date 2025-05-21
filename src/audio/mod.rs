use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Default)]
pub struct Audio {
    stream_handle: Option<OutputStreamHandle>,
    audio_sources: HashMap<usize, AudioSource>,
    sinks: Vec<Sink>,
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
    pub fn load_source(&mut self, source_bytes: Vec<u8>) -> usize {
        let source = AudioSource::new(source_bytes);
        let key = self.audio_sources.len();
        self.audio_sources.insert(key, source);
        key
    }
    pub fn play_sound(&mut self, source_id: usize) {
        let Some(stream_handle) = self.stream_handle.as_ref() else {
            log::warn!("Audio output unavailable, cannot play sound");
            return;
        };
        let Some(source) = self.audio_sources.get(&source_id) else {
            log::warn!("Unavailable audio source({source_id})");
            return;
        };
        let sink = Sink::try_new(&stream_handle).expect("Could not create audio sink");
        sink.append(source.decoder());
        sink.play();
        self.sinks.push(sink);
    }
    pub fn clean_finished_sink(&mut self) {
        let mut i = 0;
        while i < self.sinks.len() {
            if self.sinks[i].empty() {
                self.sinks.swap_remove(i);
            } else {
                i += 1;
            }
        }
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
