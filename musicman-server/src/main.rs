use musicman_protocol::{PlaylistRequest, PlaylistResponse, Request, Response};
use serde::{Deserialize, Serialize};
use std::{fs::File, sync::Arc};
use tokio::net::TcpListener;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:4000").await?;
    tracing::info!("Server listening on 0.0.0.0:4000");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        tracing::info!("New client: {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                tracing::error!("Error with client {:?}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_client(socket: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            break; // client closed
        }

        // Deserialize request
        let request: Request = bincode::deserialize(&buf[..n])?;

        tracing::info!("Requested: {:?}", request);
        match request {
            Request::Play { track_id } => {
                let file = utils::get_track_file(track_id).await?;
                stream_file(file, track_id, socket).await?;
            }
            Request::Seek { position } => {}
            Request::Playlist(plreq) => match plreq {
                PlaylistRequest::List => {
                    let playlists = get_all_playlists().await?;
                    let plres = PlaylistResponse::Playlists(playlists);
                }
                PlaylistRequest::Get { playlist_id } => {
                    let songs = utils::get_playlist(playlist_id).await?;
                    let plres = PlaylistResponse::Songs(songs);
                }
            },
            Request::Meta { track_id } => {}
        }
    }

    Ok(())
}
