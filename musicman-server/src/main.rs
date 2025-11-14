use std::sync::Arc;

use musicman_protocols::{PlaylistRequest, PlaylistResponse, Request, Response};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, tcp::OwnedReadHalf},
    sync::{Mutex, mpsc},
};

mod handlers;
mod helpers;
mod types;
use tracing::info;
use types::State;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:4000").await?;
    tracing::info!("Started index generation.");

    helpers::generate_index(&dirs::home_dir().unwrap().join("Music")).await?;
    tracing::info!("Server listening on 0.0.0.0:4000");

    loop {
        let (socket, addr) = listener.accept().await?;
        tracing::info!("New client: {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                tracing::error!("Error with client {:?}: {:?}", addr, e);
            }
        });
    }
}

async fn read_request(read: &mut OwnedReadHalf) -> anyhow::Result<Request> {
    let mut len_buf = [0u8; 4];
    read.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; msg_len];
    read.read_exact(&mut buf).await?;
    let req: Request = bincode::deserialize(&buf)?;
    Ok(req)
}

async fn handle_client(socket: tokio::net::TcpStream) -> anyhow::Result<()> {
    let (mut read, write) = socket.into_split();
    let write = Arc::new(Mutex::new(write));

    let index = helpers::load_index().await?;
    let mut state = State {
        current_stream_cancel: None,
    };

    loop {
        // Deserialize request
        let maybe_request = read_request(&mut read).await;
        if maybe_request.is_err() {
            break;
        }
        let request = maybe_request.unwrap();

        tracing::info!("Requested: {:?}", request);
        match request {
            Request::Play { track_id } => {
                if let Some(_) = &state.current_stream_cancel {
                    state.current_stream_cancel = None;
                }
                match helpers::get_track_file(&track_id, &index).await {
                    Ok(file) => {
                        let (cancel_tx, cancel_rx) = mpsc::channel::<()>(4);
                        let write_copy = write.clone();
                        tokio::spawn(async move {
                            tracing::info!("Started streaming.");
                            if let Err(e) = handlers::stream_file(
                                file,
                                track_id.clone(),
                                &write_copy,
                                cancel_rx,
                            )
                            .await
                            {
                                tracing::error!("Streaming file failed. {e}")
                            }
                        });
                        state.current_stream_cancel = Some(cancel_tx);
                    }
                    Err(e) => {
                        let res = Response::Error {
                            message: "Track not found".to_string(),
                        };
                        tracing::warn!("{e}");
                        helpers::send_to_client(&write, &res).await?;
                    }
                };
            }
            Request::Search(s_type) => {
                let data = handlers::handle_search(s_type, &index).await;
                let res = Response::SearchResults(data);
                helpers::send_to_client(&write, &res).await?;
            }
            Request::Playlist(plreq) => match plreq {
                PlaylistRequest::List => {
                    let playlists = helpers::get_all_playlists().await?;
                    let plres = PlaylistResponse::Playlists(playlists);
                    let res = Response::Playlist(plres);
                    helpers::send_to_client(&write, &res).await?;
                }
                PlaylistRequest::Get { name } => {
                    let pl = helpers::get_playlist(name).await?;
                    let plres = PlaylistResponse::Songs(pl.songs);
                    let res = Response::Playlist(plres);
                    helpers::send_to_client(&write, &res).await?;
                }
                PlaylistRequest::Create { name, songs } => {
                    helpers::create_playlist(name, songs).await?;
                }
            },
            Request::Meta { track_id } => {
                if let Some(meta) = helpers::get_track_meta(&track_id, &index).await? {
                    let res = Response::Meta(meta);
                    helpers::send_to_client(&write, &res).await?;
                } else {
                    let res = Response::Error {
                        message: "Track not found".to_string(),
                    };
                    helpers::send_to_client(&write, &res).await?;
                }
            }
        };
    }

    info!("Client Disconnected.");
    Ok(())
}
