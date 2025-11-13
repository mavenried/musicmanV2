use crate::State;
use colored::Colorize;
use std::sync::{Arc, Mutex};

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
