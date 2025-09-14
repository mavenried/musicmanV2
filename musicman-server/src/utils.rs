use musicman_protocol::SongMeta;
use std::collections::HashMap;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

async fn stream_file(
    mut file: tokio::fs::File,
    track_id: String,
    mut stream: TcpStream,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut index: u32 = 0;

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            let res = Response::EndOfStream;
            send_response(&mut stream, &res).await?;
            break;
        }

        let res = Response::SongChunk {
            track_id: track_id.clone(),
            data: buf[..n].to_vec(),
            index,
        };

        send_to_client(&mut stream, &res).await?;
        index += 1;
    }

    Ok(())
}

pub type SongIndex = HashMap<Uuid, SongMeta>;

pub async fn load_index() -> anyhow::Result<SongIndex> {
    let index_file = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("index.json");
    let data = tokio::fs::read_to_string(index_file).await?;
    let index: SongIndex = serde_json::from_str(&data)?;
    Ok(index)
}

pub async fn save_index(index: &SongIndex) -> anyhow::Result<()> {
    let index_file = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("index.json");
    let data = serde_json::to_string_pretty(index)?;
    tokio::fs::write(index_file, data).await?;
    Ok(())
}

pub async fn get_track_file(track_id: String) -> anyhow::Result<File> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
        .unwrap()
        .join("musicman")
        .join("tracks")
        .join(format!("{}.mp3", track_id));

    let file = OpenOptions::new()
        .read(true)
        .open(config_dir)
        .await
        .unwrap();

    Ok(file)
}

pub async fn get_playlist(playlist_id: String) -> anyhow::Result<Vec<String>> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("playlists")
        .join(format!("{}.json", playlist_id));

    let data = tokio::fs::read_to_string(config_dir).await?;
    let songs: Vec<String> = serde_json::from_str(&data)?;
    Ok(songs)
}

pub async fn get_all_playlists() -> anyhow::Result<Vec<String>> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("playlists");
    let mut playlists = vec![];
    let mut dir = tokio::fs::read_dir(config_dir).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                playlists.push(name.to_string());
            }
        }
    }
    Ok(playlists)
}

async fn send_to_client(
    socket: &mut tokio::net::TcpStream,
    response: &Response,
) -> anyhow::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(response)?;
    socket.write_all(&encoded).await?;
    Ok(())
}
