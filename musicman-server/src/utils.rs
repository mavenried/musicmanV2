use musicman_protocol::*;
use std::collections::HashMap;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::Uuid;

use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions,
};

pub async fn stream_file(
    file: tokio::fs::File,
    track_id: String,
    stream: &mut TcpStream,
) -> anyhow::Result<()> {
    // Convert tokio file → std file (Symphonia needs std::io::Read + Seek).
    let std_file = file.into_std().await;
    let mss = MediaSourceStream::new(Box::new(std_file), Default::default());

    // Probe the file format.
    let probed = symphonia::default::get_probe().format(
        &Default::default(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    // Pick the first audio track.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec.is_some())
        .ok_or_else(|| anyhow::anyhow!("No supported audio tracks"))?;

    let dec_opts = DecoderOptions { verify: true };
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;

    let mut index: u32 = 0;

    // Read packets.
    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(symphonia::core::errors::Error::IoError(_)) => break, // EOF
            Err(e) => return Err(e.into()),
        };

        // Decode packet.
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        };

        // Convert decoded buffer → interleaved i16 samples.
        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<i16>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        let samples = sample_buf.samples();

        // Chunk into ~4KB pieces.
        for chunk in samples.chunks(2048) {
            let data: Vec<i16> = chunk.to_vec();

            let res = Response::SongChunk {
                track_id: track_id.clone(),
                data,
                index,
            };

            send_to_client(stream, &res).await?;
            index += 1;
        }
    }

    // End marker.
    let res = Response::EndOfStream;
    send_to_client(stream, &res).await?;

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

    let file = OpenOptions::new().read(true).open(config_dir).await?;

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

pub async fn get_track_meta(track_id: &str) -> anyhow::Result<Option<SongMeta>> {
    let index = load_index().await?;
    if let Some(meta) = index.get(&Uuid::parse_str(track_id)?) {
        Ok(Some(meta.clone()))
    } else {
        Ok(None)
    }
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
        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && let Some(name) = path.file_stem().and_then(|s| s.to_str())
        {
            playlists.push(name.to_string());
        }
    }
    Ok(playlists)
}

pub async fn send_to_client(
    socket: &mut tokio::net::TcpStream,
    response: &Response,
) -> anyhow::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(response)?;
    socket.write_all(&encoded).await?;
    Ok(())
}
