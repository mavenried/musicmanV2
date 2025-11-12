use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum Request {
    Play { track_id: Uuid }, // param uuid
    Seek { position: u64 },
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
    SongChunk {
        track_id: Uuid,
        data: Vec<i16>,
        index: u32,
    },
    EndOfStream,
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
