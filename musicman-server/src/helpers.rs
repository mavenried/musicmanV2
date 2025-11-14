use crate::types::*;
use anyhow::Result;
use musicman_protocols::*;
use std::{collections::HashMap, path::PathBuf};
use symphonia::{
    core::{
        formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, units::TimeStamp,
    },
    default::get_probe,
};
use tokio::{
    fs::{self, File, OpenOptions},
    io::AsyncWriteExt,
};
use uuid::Uuid;
use walkdir::WalkDir;

pub async fn send_to_client(socket: &WriteSocket, response: &Response) -> anyhow::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(response)?;
    let len = (encoded.len() as u32).to_be_bytes();
    let mut socket_locked = socket.lock().await;
    socket_locked.write_all(&len).await?;
    socket_locked.write_all(&encoded).await?;
    Ok(())
}

pub async fn generate_index(music_dir: &PathBuf) -> anyhow::Result<()> {
    // collect supported audio files
    let mut songs: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(music_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp3" | "flac" | "wav" | "ogg" | "m4a" => songs.push(path.to_path_buf()),
                _ => {}
            }
        }
    }

    tracing::info!("Found {} songs.", songs.len());

    let mut index: SongIndex = HashMap::new();

    for path in songs {
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Skipping {:?}: open error: {}", path, e);
                continue;
            }
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut probe = match get_probe().format(
            &Default::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        ) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Skipping {:?}: probe error: {}", path, e);
                continue;
            }
        };

        let mut format = probe.format;

        // Defaults
        let mut title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let mut artist = "Unknown".to_string();
        let mut duration_secs: u32 = 0;
        let mut meta_opt = format.metadata();

        if meta_opt.current().is_none() {
            if let Some(meta) = probe.metadata.get() {
                meta_opt = meta;
            }
        }
        if let Some(rev) = meta_opt.current() {
            // rev.tags() returns an iterator of tags; tag.key and tag.value are Options
            for tag in rev.tags() {
                let key = tag.key.to_string();
                let val = tag.value.to_string();
                match key.to_lowercase().as_str() {
                    "title" | "tit2" if !val.is_empty() => title = val.to_string(),
                    "artist" | "tpe1" if !val.is_empty() => artist = val.to_string(),
                    _ => {}
                }
            }
        }

        if let Some(track) = format.tracks().first() {
            if let (Some(tb), Some(n_frames)) =
                (track.codec_params.time_base, track.codec_params.n_frames)
            {
                let ts: TimeStamp = n_frames as TimeStamp;
                let time = tb.calc_time(ts); // has .seconds (u64) and .frac (f64)
                let secs_f = (time.seconds as f64) + time.frac;
                duration_secs = secs_f.max(0.0).round() as u32;
            }
        }

        let id = uuid::Uuid::new_v5(&Uuid::NAMESPACE_URL, path.display().to_string().as_bytes());
        let artists = artist
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let songmeta = SongMeta {
            id,
            title,
            artists,
            duration: duration_secs,
            path,
        };

        index.insert(id, songmeta);
    }

    tracing::info!("Indexed {} songs.", index.len());
    save_index(&index).await?;
    Ok(())
}

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

pub async fn get_track_file(track_id: &Uuid, index: &SongIndex) -> anyhow::Result<File> {
    if let Some(meta) = get_track_meta(track_id, index).await? {
        let track_path = meta.path;
        let file = OpenOptions::new().read(true).open(track_path).await?;
        return Ok(file);
    }
    Err(anyhow::anyhow!("File Not Found!"))
}

pub async fn get_track_meta(
    track_id: &Uuid,
    index: &SongIndex,
) -> anyhow::Result<Option<SongMeta>> {
    if let Some(meta) = index.get(track_id) {
        Ok(Some(meta.clone()))
    } else {
        Ok(None)
    }
}

fn sanitize(name: String) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub async fn get_playlist(name: String) -> Result<PlaylistMeta> {
    let playlists_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("playlists");

    let file_path = playlists_dir.join(format!("{}.json", sanitize(name)));
    let data = fs::read_to_string(file_path).await?;
    let playlist: PlaylistMeta = serde_json::from_str(&data)?;
    Ok(playlist)
}

pub async fn get_all_playlists() -> Result<Vec<Playlist>> {
    let playlists_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("playlists");

    let mut dir = fs::read_dir(playlists_dir).await?;
    let mut result = vec![];

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = fs::read_to_string(&path).await?;
            let playlist: PlaylistMeta = serde_json::from_str(&data)?;
            result.push(Playlist {
                name: playlist.title,
                len: playlist.songs.len(),
            });
        }
    }

    Ok(result)
}

pub async fn create_playlist(title: String, songs: Vec<SongMeta>) -> Result<()> {
    let playlists_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("musicman")
        .join("playlists");

    fs::create_dir_all(&playlists_dir).await?;

    let playlist = PlaylistMeta {
        title: sanitize(title),
        songs,
    };

    let file_path = playlists_dir.join(format!("{}.json", playlist.title));
    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(file_path, data).await?;

    Ok(())
}
