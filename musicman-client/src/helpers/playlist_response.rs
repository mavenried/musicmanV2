use crate::{helpers, types::*};
use musicman_protocol::*;
use std::{net::TcpStream, sync::mpsc};
use tabled::settings::disable::Remove;
use tabled::settings::{Style, object::Columns};

pub fn handle_playlist_response(
    plres: PlaylistResponse,
    stream: &TcpStream,
    state: &ClientState,
    utx: &mpsc::Sender<UiRequest>,
    srx: &mpsc::Receiver<UiResponse>,
) {
    match plres {
        PlaylistResponse::Playlists(playlists) => {
            if playlists.len() == 0 {
                utx.send(UiRequest::Display(format!("No playlists yet.")))
                    .unwrap();
                return;
            }

            let mut i = 0;
            let table_vec = playlists.iter().map(|pl| {
                i += 1;
                PlaylistTable {
                    id: i,
                    name: pl.name.clone(),
                    length: pl.len,
                }
            });
            let out = tabled::Table::new(table_vec)
                .with(Style::rounded())
                .to_string();
            utx.send(UiRequest::Display(out)).unwrap();
        }
        PlaylistResponse::Songs(songs) => {
            let len = songs.len();

            if len == 0 {
                utx.send(UiRequest::Display(format!("No results.")))
                    .unwrap();
                return;
            }

            let mut i = 0;
            let table_vec = songs
                .iter()
                .map(|sm| {
                    i += 1;
                    let mins = sm.duration / 60;
                    let secs = sm.duration % 60;
                    SongTable {
                        id: i,
                        title: sm.title.clone(),
                        artists: sm.artists.join(", "),
                        duration: format!("{}m{}s", mins, secs),
                        playing: PlayingDisplay(false),
                    }
                })
                .collect::<Vec<SongTable>>();
            let out = tabled::Table::new(table_vec)
                .with(Style::rounded())
                .with(Remove::column(Columns::one(1)))
                .to_string();
            utx.send(UiRequest::Prompt {
                s: out,
                prompt: "Replace queue? (y/N)".to_string(),
            })
            .unwrap();

            if let Ok(s) = srx.recv() {
                if s.0.starts_with("y") {
                    if let Ok(mut s) = state.lock() {
                        s.current_song = Some(songs[0].clone());
                        s.queue = songs.clone();
                        s.current_idx = 0;
                    }

                    utx.send(UiRequest::Display(format!("Added {len} songs to queue.")))
                        .unwrap();
                    helpers::send_to_server(
                        stream,
                        Request::Play {
                            track_id: songs[0].id,
                        },
                    );
                }
            }
        }
    }
}
