use crate::{helpers, types::*};
use colored::Colorize;
use musicman_protocols::*;
use std::net::TcpStream;

mod next_prev;
mod playlist;
mod show;
pub use next_prev::*;
pub use playlist::*;
pub use show::*;

pub fn print_help() {
    println!("{}", "Valid Commands =>".yellow());
    println!("  {}", "clear        => Clear queue. ".blue());
    println!("  {}", "show, ls     => Show Queue.".blue());
    println!("  {}", "search       => Search index. ".blue());
    println!("  {}", "replay       => Replay current song.".blue());
    println!("  {}", "pause, p     => Toggle pause/play.".blue());
    println!("  {}", "next         => Goto next song.".blue());
    println!("  {}", "prev         => Goto previous song.".blue());
    println!("  {}", "playlist, pl => Playlist management.".blue());
    println!("  {}", "exit         => Exit the player.".blue());
}

pub fn handle_replay(stream: &TcpStream, state: &ClientState) {
    let song = state.lock().unwrap().current_song.clone();
    if song.is_none() {
        println!("{}", "No song to replay".red());
    } else {
        helpers::send_to_server(
            &stream,
            Request::Play {
                track_id: song.clone().unwrap().id,
            },
        );
        println!("{} {}", "Replaying:".yellow(), song.unwrap().title.blue());
    };
}

pub fn handle_pause(sink: &RodioSink) {
    if let Ok(sink) = sink.lock() {
        if sink.is_paused() {
            sink.play();
            println!("{}", "Playing...".green());
        } else {
            sink.pause();
            println!("{}", "Paused.".green());
        };
    }
}

pub fn handle_clear(state: &ClientState, sink: &RodioSink) {
    println!("{}", "Clearing queue...".green());
    if let Ok(mut s) = state.lock() {
        s.queue.clear();
        s.current_song = None;
    }
    sink.lock().unwrap().clear();
}

pub fn handle_search(stream: &TcpStream, input: Vec<String>) {
    if input.len() >= 2 {
        match input[1].as_str() {
            "artist" | "a" => {
                let artist = input[2..].join(" ").to_lowercase();
                helpers::send_to_server(&stream, Request::Search(SearchType::ByArtist(artist)));
                return;
            }
            "title" | "t" => {
                let title = input[2..].join(" ").to_lowercase();
                helpers::send_to_server(&stream, Request::Search(SearchType::ByTitle(title)));
                return;
            }
            _ => (),
        }
    }
    println!(
        "{}",
        format!(
            "Usage: {} [{}] <{}>",
            "search".blue().bold(),
            "selector".yellow(),
            "query".purple()
        )
        .red()
    );
    println!(
        "{}",
        "selector =>
  a | artist [search by artist]
  t | title  [search by title]
"
        .yellow()
    );
}
