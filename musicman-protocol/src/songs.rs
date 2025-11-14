use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: u32, // in seconds
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Eq)]
pub enum SearchType {
    ByTitle(String),
    ByArtist(String),
}
