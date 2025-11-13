use crate::{helpers, types::*};
use colored::Colorize;
use musicman_protocol::*;
use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread,
};

type Stream = TcpStream;

pub fn server_interface(
    mut stream: Stream,
    ptx: Sender<Response>,
    utx: Sender<UiRequest>,
    srx: Receiver<UiResponse>,
) {
    thread::spawn(move || {
        loop {
            if let Ok(response) = helpers::read_from_client(&mut stream) {
                match response {
                    Response::Meta(songmeta) => {
                        let mins = songmeta.duration / 60;
                        let secs = songmeta.duration % 60;
                        utx.send(UiRequest::Display(format!(
                            "❯ {} by {} | {}m {}s",
                            songmeta.title,
                            songmeta.artists.join(", "),
                            mins,
                            secs
                        )))
                        .unwrap();
                    }
                    Response::SongChunk { .. }
                    | Response::SongHeader { .. }
                    | Response::EndOfStream { .. } => {
                        ptx.send(response).unwrap();
                    }
                    Response::Playlist(plres) => match plres {
                        PlaylistResponse::Playlists(playlists) => {}
                        PlaylistResponse::Songs(songs) => {
                            if songs.len() > 100 {
                                utx.send(UiRequest::Prompt {
                                    s: "Over 100 results found. Show all?".to_string(),
                                    prompt: "choice".to_string(),
                                })
                                .unwrap();
                                if let Ok(res) = srx.recv() {
                                    if !res.0.starts_with('y') {
                                        continue;
                                    }
                                }
                            }
                            utx.send(UiRequest::GetMeta(songs)).unwrap();
                        }
                    },

                    _ => {
                        println!("Received response: {:?}", response);
                    }
                }
            } else {
                println!("{}", "Error: Failed to deserialize response".red());
            }
        }
    });
}
