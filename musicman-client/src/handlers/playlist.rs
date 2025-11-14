use crate::{helpers, types::ClientState};
use colored::Colorize;
use musicman_protocol::{PlaylistRequest, Request};
use std::net::TcpStream;

pub fn handle_playlist(stream: &TcpStream, input: Vec<String>, state: &ClientState) {
    if input.len() >= 2 {
        match input[1].as_str() {
            "load" | "new" => {
                if input.len() >= 3 {
                    let name = input[2..].join(" ").to_lowercase();
                    if input[1] == "load" {
                        println!("{}", format!("Loading playlist: {}", name).green());
                        helpers::send_to_server(
                            stream,
                            Request::Playlist(PlaylistRequest::Get { name }),
                        );
                    } else if input[1] == "new" {
                        println!("{}", format!("Creating new playlist: {}", name).green());
                        helpers::send_to_server(
                            stream,
                            Request::Playlist(PlaylistRequest::Create {
                                name,
                                songs: state.lock().unwrap().queue.clone(),
                            }),
                        );
                    }
                    return;
                }
            }
            "ls" | "show" => {
                helpers::send_to_server(stream, Request::Playlist(PlaylistRequest::List));
                return;
            }
            _ => (),
        }
    }

    println!(
        "{}",
        format!(
            "Usage: {} [{}] <{}>",
            "pl".blue().bold(),
            "operation".yellow(),
            "name".purple()
        )
        .red()
    );
    println!(
        "{}",
        "operation =>
  load [load a playlist by name]
  new  [make new playlist with current queue named <name>]
  show [list available playlists]
"
        .yellow()
    );
}
