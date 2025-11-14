use crate::types::*;
use musicman_protocols::*;
use rodio::buffer::SamplesBuffer;
use std::{sync::mpsc::Receiver, thread};

pub fn player(prx: Receiver<Response>, sink: RodioSink, player_state: PlayerState) {
    thread::spawn(move || {
        loop {
            let res = match prx.recv() {
                Ok(r) => r,
                Err(_) => continue, // channel closed? just loop
            };

            let mut ps = match player_state.lock() {
                Ok(l) => l,
                Err(_) => continue,
            };

            match res {
                Response::SongHeader {
                    sample_rate: sr,
                    channels: ch,
                    track_id,
                } => {
                    ps.channels = ch;
                    ps.sample_rate = sr;
                    ps.current_id = Some(track_id);
                    ps.waiting_for_header = false;

                    if let Ok(s) = sink.lock() {
                        s.clear();
                        s.play();
                    }
                }

                Response::SongChunk { data, track_id, .. } => {
                    if ps.current_id != Some(track_id) {
                        continue;
                    }
                    if let Ok(s) = sink.lock() {
                        let src = SamplesBuffer::new(ps.channels, ps.sample_rate, data);
                        s.append(src);
                    }
                }

                Response::EndOfStream { track_id } => {
                    if ps.current_id == Some(track_id) {
                        ps.current_id = None;
                    }
                }

                _ => {}
            }
        }
    });
}
