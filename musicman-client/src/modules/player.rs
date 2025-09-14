use crate::State;
use colored::Colorize;
use std::io::Read;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

struct ChannelReader {
    rx: mpsc::Receiver<Vec<u8>>,
    buffer: Vec<u8>,
}

impl Read for ChannelReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            match self.rx.recv() {
                Ok(chunk) => self.buffer = chunk,
                Err(_) => return Ok(0), // channel closed => EOF
            }
        }
        let n = out.len().min(self.buffer.len());
        out[..n].copy_from_slice(&self.buffer[..n]);
        self.buffer.drain(..n);
        Ok(n)
    }
}

pub fn player(sink: Arc<rodio::Sink>, prx: mpsc::Receiver<Vec<u8>>) {
    std::thread::spawn(|| {
        let mut reader = ChannelReader {
            rx: prx,
            buffer: Vec::new(),
        };
        let source = rodio::Decoder::new(reader).unwrap();
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
    if state.songid.is_empty() {
        state.songid = state.queue[0].clone();
        println!("{}", format!("Now playing: {}", state.songid).green());
        return;
    }
    let current_index = state
        .queue
        .iter()
        .position(|x| *x == state.songid)
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
    state.songid = state.queue[new_index].clone();
    println!("{}", format!("Now playing: {}", state.songid).green());
}

pub fn get_prev_song(state: &Arc<Mutex<State>>, n: usize) {
    let mut state = state.lock().unwrap();
    if state.queue.is_empty() {
        println!("{}", "Queue is empty!".red());
        return;
    }
    if state.songid.is_empty() {
        state.songid = state.queue[0].clone();
        println!("{}", format!("Now playing: {}", state.songid).green());
        return;
    }
    let current_index = state
        .queue
        .iter()
        .position(|x| *x == state.songid)
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
    state.songid = state.queue[new_index].clone();
    println!("{}", format!("Now playing: {}", state.songid).green());
}
