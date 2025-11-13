use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Request {
    Play { track_id: Uuid }, // param uuid
    Playlist(PlaylistRequest),
    Meta { track_id: Uuid },
    Search(SearchType),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum SearchType {
    ByTitle(String),
    ByArtist(String),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: u32, // in seconds
    pub path: PathBuf,
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
    Meta(SongMeta),
    Error {
        message: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistRequest {
    Get { playlist_id: Uuid }, // param uuid
    List,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistResponse {
    Playlists(Vec<Uuid>),
    Songs(Vec<Uuid>),
}
