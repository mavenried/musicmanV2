use musicman_protocol::{PlaylistRequest, PlaylistResponse, Request, Response};
use tokio::{io::AsyncReadExt, net::TcpListener};

mod handlers;
mod helpers;

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

async fn read_request(socket: &mut tokio::net::TcpStream) -> anyhow::Result<Request> {
    let mut len_buf = [0u8; 4];
    socket.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; msg_len];
    socket.read_exact(&mut buf).await?;
    let req: Request = bincode::deserialize(&buf)?;
    Ok(req)
}
async fn handle_client(mut socket: tokio::net::TcpStream) -> anyhow::Result<()> {
    let index = helpers::load_index().await?;

    loop {
        // Deserialize request
        let maybe_request = read_request(&mut socket).await;
        if maybe_request.is_err() {
            break;
        }
        let request = maybe_request.unwrap();

        tracing::info!("Requested: {:?}", request);
        match request {
            Request::Play { track_id } => match helpers::get_track_file(&track_id, &index).await {
                Ok(file) => handlers::stream_file(file, &track_id, &mut socket).await?,
                Err(e) => {
                    let res = Response::Error {
                        message: "Track not found".to_string(),
                    };
                    tracing::warn!("{e}");
                    helpers::send_to_client(&mut socket, &res).await?;
                }
            },
            Request::Search(s_type) => {
                let data = handlers::handle_search(s_type, &index).await;
                let res = Response::Playlist(PlaylistResponse::Songs(data));
                helpers::send_to_client(&mut socket, &res).await?;
            }
            Request::Playlist(plreq) => match plreq {
                PlaylistRequest::List => {
                    let playlists = helpers::get_all_playlists().await?;
                    let plres = PlaylistResponse::Playlists(playlists);
                    let res = Response::Playlist(plres);
                    helpers::send_to_client(&mut socket, &res).await?;
                }
                PlaylistRequest::Get { playlist_id } => {
                    let songs = helpers::get_playlist(&playlist_id).await?;
                    let plres = PlaylistResponse::Songs(songs);
                    let res = Response::Playlist(plres);
                    helpers::send_to_client(&mut socket, &res).await?;
                }
            },
            Request::Meta { track_id } => {
                if let Some(meta) = helpers::get_track_meta(&track_id, &index).await? {
                    let res = Response::Meta(meta);
                    helpers::send_to_client(&mut socket, &res).await?;
                } else {
                    let res = Response::Error {
                        message: "Track not found".to_string(),
                    };
                    helpers::send_to_client(&mut socket, &res).await?;
                }
            }
        };
    }

    Ok(())
}
