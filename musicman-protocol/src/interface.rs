use crate::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Request {
    Play { track_id: Uuid }, // param uuid
    Playlist(PlaylistRequest),
    Meta { track_id: Uuid },
    Search(SearchType),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Response {
    SongHeader {
        track_id: Uuid,
        channels: u16,
        sample_rate: u32,
    },
    SongChunk {
        track_id: Uuid,
        data: Vec<i16>,
        index: u32,
    },
    EndOfStream {
        track_id: Uuid,
    },

    Playlist(PlaylistResponse),
    SearchResults(Vec<SongMeta>),
    Meta(SongMeta),
    Error {
        message: String,
    },
}
