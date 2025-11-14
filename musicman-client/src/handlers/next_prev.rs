use crate::{helpers, player, types::*};
use colored::Colorize;
use musicman_protocol::Request;
use std::net::TcpStream;

pub fn handle_next_prev(stream: &TcpStream, state: &ClientState, input: Vec<String>) {
    let is_next = input[0] == "next";

    let n = match input.get(1) {
        Some(val) => match val.parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                println!(
                    "{}",
                    format!("Usage: {} <number>", input[0].blue().bold()).red()
                );
                return;
            }
        },
        None => 1,
    };

    let result = if is_next {
        player::get_next_song(&state, n)
    } else {
        player::get_prev_song(&state, n)
    };

    if result == GetReturn::QueueEmpty {
        println!("{}", "Queue Empty!".red());
        return;
    }

    if let Some(song) = state.lock().unwrap().current_song.clone() {
        println!("{} {}", "Playing:".yellow(), song.title.blue());

        helpers::send_to_server(stream, Request::Play { track_id: song.id });
    }
}
