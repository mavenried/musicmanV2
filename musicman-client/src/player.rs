use crate::State;
use colored::Colorize;
use musicman_protocol::*;
use rodio::{OutputStream, Sink};
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub fn player(prx: mpsc::Receiver<Response>, sink: Arc<Sink>) {
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

pub fn get_next_song(state: &Arc<Mutex<State>>, n: usize) {
    let mut state = state.lock().unwrap();
    if state.queue.is_empty() {
        println!("{}", "Queue is empty!".red());
        return;
    }
    if state.songid.is_none() {
        state.songid = Some(state.queue[0].clone());
        println!(
            "{}",
            format!("Now playing: {}", state.songid.unwrap()).green()
        );
        return;
    }
    let current_index = state
        .queue
        .iter()
        .position(|x| *x == state.songid.unwrap())
        .unwrap_or(0);
    if current_index >= state.queue.len() - 1 {
        println!("{}", "Already at the end of the queue!".red());
        return;
    }
    let new_index = if current_index + n < state.queue.len() {
        current_index + n
    } else {
        state.queue.len() - 1
    };
    state.songid = Some(state.queue[new_index].clone());
    println!(
        "{}",
        format!("Now playing: {}", state.songid.unwrap()).green()
    );
}

pub fn get_prev_song(state: &Arc<Mutex<State>>, n: usize) {
    let mut state = state.lock().unwrap();
    if state.queue.is_empty() {
        println!("{}", "Queue is empty!".red());
        return;
    }
    if state.songid.is_none() {
        state.songid = Some(state.queue[0].clone());
        println!(
            "{}",
            format!("Now playing: {}", state.songid.unwrap()).green()
        );
        return;
    }
    let current_index = state
        .queue
        .iter()
        .position(|x| *x == state.songid.unwrap())
        .unwrap_or(0);
    if current_index == 0 {
        println!("{}", "Already at the beginning of the queue!".red());
        return;
    }
    let new_index = if current_index >= n {
        current_index - n
    } else {
        0
    };
    state.songid = Some(state.queue[new_index].clone());
    println!(
        "{}",
        format!("Now playing: {}", state.songid.unwrap()).green()
    );
}
