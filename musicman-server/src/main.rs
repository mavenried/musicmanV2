use musicman_protocol::{PlaylistRequest, PlaylistResponse, Request, Response};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:4000").await?;
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

async fn handle_client(mut socket: tokio::net::TcpStream) -> anyhow::Result<()> {
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
                if let Ok(file) = utils::get_track_file(track_id.clone()).await {
                    utils::stream_file(file, track_id, &mut socket).await?;
                } else {
                    let res = Response::Error {
                        message: "Track not found".to_string(),
                    };
                    tracing::warn!("Track not found");
                    utils::send_to_client(&mut socket, &res).await?;
                }
            }
            Request::Seek { position } => {}
            Request::Playlist(plreq) => match plreq {
                PlaylistRequest::List => {
                    let playlists = utils::get_all_playlists().await?;
                    let plres = PlaylistResponse::Playlists(playlists);
                }
                PlaylistRequest::Get { playlist_id } => {
                    let songs = utils::get_playlist(playlist_id).await?;
                    let plres = PlaylistResponse::Songs(songs);
                }
            },
            Request::Meta { track_id } => {
                if let Some(meta) = utils::get_track_meta(&track_id).await? {
                    let res = Response::Meta(meta);
                    utils::send_to_client(&mut socket, &res).await?;
                } else {
                    let res = Response::Error {
                        message: "Track not found".to_string(),
                    };
                    utils::send_to_client(&mut socket, &res).await?;
                }
            }
        };
    }

    Ok(())
}
