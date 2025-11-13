use musicman_protocol::*;
use rodio::{OutputStream, Sink};
use std::{
    sync::{Arc, mpsc::Receiver},
    thread,
};

pub fn player(prx: Receiver<Response>, sink: Arc<Sink>) {
    thread::spawn(move || {
        let mut samples: Vec<i16> = Vec::new();

        // Collect all audio chunks first
        while let Ok(msg) = prx.recv() {
            match msg {
                Response::SongChunk { data, .. } => {
                    samples.extend_from_slice(&data);
                }
                Response::EndOfStream { track_id: _ } => {
                    break;
                }
                Response::Error { message } => {
                    eprintln!("Server error: {}", message);
                    return;
                }
                _ => {}
            }
        }

        if samples.is_empty() {
            eprintln!("No audio data received.");
            return;
        }

        // Playback once fully buffered
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Audio output error: {}", e);
                return;
            }
        };

        // convert Vec<i16> -> rodio Source
        let source = rodio::buffer::SamplesBuffer::new(2, 44100, samples);
        sink.append(source);
        sink.sleep_until_end();
    });
}
