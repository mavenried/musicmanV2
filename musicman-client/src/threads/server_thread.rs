use crate::{helpers, types::*};
use musicman_protocols::*;
use std::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread,
};
use tabled::settings::{Remove, Style, object::Columns};

type Stream = TcpStream;

pub fn server_interface(
    mut stream: Stream,
    state: ClientState,
    ptx: Sender<Response>,
    utx: Sender<UiRequest>,
    srx: Receiver<UiResponse>,
) {
    thread::spawn(move || {
        loop {
            match helpers::read_from_client(&mut stream) {
                Ok(response) => match response {
                    Response::Meta(songmeta) => {
                        let mins = songmeta.duration / 60;
                        let secs = songmeta.duration % 60;
                        let data = SongTable {
                            id: 1,
                            title: songmeta.title,
                            artists: songmeta.artists.join(", "),
                            duration: format!("{mins}m{secs}s"),
                            playing: PlayingDisplay(false),
                        };
                        utx.send(UiRequest::Display(
                            tabled::Table::new(vec![data])
                                .with(Style::rounded())
                                .with(Remove::column(Columns::one(1)))
                                .to_string(),
                        ))
                        .unwrap();
                    }
                    Response::SongChunk { .. }
                    | Response::SongHeader { .. }
                    | Response::EndOfStream { .. } => {
                        ptx.send(response).unwrap();
                    }
                    Response::Playlist(plres) => {
                        helpers::handle_playlist_response(plres, &stream, &state, &utx, &srx)
                    }
                    Response::SearchResults(data) => {
                        helpers::handle_search_response(data, &stream, &state, &utx, &srx)
                    }
                    Response::Error { message } => {
                        utx.send(UiRequest::Display(message)).unwrap();
                    }
                },
                Err(e) => {
                    if let Some(ioerr) = e.downcast_ref::<io::Error>() {
                        if ioerr.kind() == io::ErrorKind::UnexpectedEof {
                            utx.send(UiRequest::Shutdown).unwrap();
                            break;
                        } else {
                            utx.send(UiRequest::Display("Error: {e}".to_string()))
                                .unwrap();
                        }
                    }
                }
            }
        }
    });
}
