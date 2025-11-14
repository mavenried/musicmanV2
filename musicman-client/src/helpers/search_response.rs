use crate::{helpers, types::*};
use musicman_protocols::*;
use std::{net::TcpStream, sync::mpsc};
use tabled::settings::disable::Remove;
use tabled::settings::{Style, object::Columns};

pub fn handle_search_response(
    songs: Vec<SongMeta>,
    stream: &TcpStream,
    state: &ClientState,
    utx: &mpsc::Sender<UiRequest>,
    srx: &mpsc::Receiver<UiResponse>,
) {
    if songs.is_empty() {
        utx.send(UiRequest::Display("No results.".to_string()))
            .unwrap();
        return;
    }
    if songs.len() > 100 {
        utx.send(UiRequest::Display(format!(
            "More than {} matches, please be more specific.",
            songs.len()
        )))
        .unwrap();
        return;
    }

    let table_vec: Vec<SongTable> = songs
        .iter()
        .enumerate()
        .map(|(i, sm)| {
            let mins = sm.duration / 60;
            let secs = sm.duration % 60;
            SongTable {
                id: (i + 1) as u16,
                title: sm.title.clone(),
                artists: sm.artists.join(", "),
                duration: format!("{}m{}s", mins, secs),
                playing: PlayingDisplay(false),
            }
        })
        .collect();

    let table_str = tabled::Table::new(table_vec)
        .with(Style::rounded())
        .with(Remove::column(Columns::one(1)))
        .to_string();

    utx.send(UiRequest::Prompt {
        s: table_str,
        prompt: "Pick (*, nums)".to_string(),
    })
    .unwrap();

    if let Ok(response) = srx.recv() {
        let input = response.0.trim();

        let selected_songs = if input == "*" {
            songs.clone()
        } else {
            let mut selected = Vec::new();
            for part in input.split(' ') {
                if let Ok(idx) = part.trim().parse::<usize>() {
                    if idx >= 1 && idx <= songs.len() {
                        selected.push(songs[idx - 1].clone());
                    }
                }
            }
            selected
        };

        if selected_songs.is_empty() {
            utx.send(UiRequest::Display("Added 0 songs to queue.".to_string()))
                .unwrap();
            return;
        }

        let is_queue_empty;
        if let Ok(mut s) = state.lock() {
            if s.queue.is_empty() {
                is_queue_empty = true;
                s.queue = selected_songs.clone();
                s.current_song = Some(selected_songs[0].clone());
                s.current_idx = 0;
            } else {
                is_queue_empty = false;
                s.queue.extend(selected_songs.clone());
            }
            if is_queue_empty {
                helpers::send_to_server(
                    stream,
                    Request::Play {
                        track_id: selected_songs[0].id,
                    },
                );
            }
        }

        utx.send(UiRequest::Display(format!(
            "Added {} songs to queue.",
            selected_songs.len()
        )))
        .unwrap();
    }
}
