use crate::helpers::*;
use musicman_protocol::*;
use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs::{CODEC_TYPE_NULL, DecoderOptions},
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::get_probe,
};
use tokio::net::TcpStream;
use uuid::Uuid;

pub async fn stream_file(
    file: tokio::fs::File,
    track_id: &Uuid,
    stream: &mut TcpStream,
) -> anyhow::Result<()> {
    let std_file = file.into_std().await;
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(std_file), Default::default());

    let probed = get_probe().format(
        &Default::default(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("No supported audio tracks"))?;

    let dec_opts = DecoderOptions { verify: true };
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;

    // Send a header first
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(1);

    let header = Response::SongHeader {
        track_id: track_id.clone(),
        channels,
        sample_rate,
    };
    send_to_client(stream, &header).await?;

    let mut index: u32 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(symphonia::core::errors::Error::IoError(_)) => break, // EOF
            Err(e) => return Err(e.into()),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        };

        let duration = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<i16>::new(duration, *decoded.spec());
        sample_buf.copy_interleaved_ref(decoded);

        let samples = sample_buf.samples();

        for chunk in samples.chunks(2000) {
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

    let res = Response::EndOfStream {
        track_id: track_id.clone(),
    };
    send_to_client(stream, &res).await?;

    Ok(())
}

pub async fn handle_search(s: SearchType, index: &SongIndex) -> Vec<Uuid> {
    let mut results = Vec::new();

    match s {
        SearchType::ByTitle(query) => {
            let q = query.to_lowercase();
            for (id, meta) in index {
                if meta.title.to_lowercase().contains(&q) {
                    results.push(*id);
                }
            }
        }
        SearchType::ByArtist(query) => {
            let q = query.to_lowercase();
            for (id, meta) in index {
                for artist in meta.artists.clone() {
                    if artist.to_lowercase().contains(&q) {
                        results.push(*id);
                    }
                }
            }
        }
    }

    results
}
