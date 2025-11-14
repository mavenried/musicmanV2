use crate::SongMeta;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub struct Playlist {
    pub name: String,
    pub len: usize,
}
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub struct PlaylistMeta {
    pub title: String,
    pub songs: Vec<SongMeta>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistRequest {
    Get { name: String },
    Create { name: String, songs: Vec<SongMeta> },
    List,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum PlaylistResponse {
    Playlists(Vec<Playlist>),
    Songs(Vec<SongMeta>),
}
