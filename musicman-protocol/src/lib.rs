use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Request {
    Play { track_id: String }, // param uuid
    Seek { position: u64 },
    Playlist(PlaylistRequest),
    Meta { track_id: String },
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub duration: u32, // in seconds
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Response {
    SongChunk {
        track_id: String,
        data: Vec<u8>,
        index: u32,
    },
    EndOfStream,
    Playlist(PlaylistResponse),
    Meta {
        track_id: String,
        title: String,
        duration: u64,
        artist: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistRequest {
    Get { playlist_id: String }, // param uuid
    List,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistResponse {
    Playlists(Vec<String>),
    Songs(Vec<String>),
}
