use musicman_protocols::SongMeta;
use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tabled::Tabled;
use uuid::Uuid;
mod linereader;
pub use linereader::*;

mod highlighter;
pub use highlighter::*;

pub struct ClientStateStruct {
    pub current_song: Option<SongMeta>,
    pub queue: Vec<SongMeta>,
    pub current_idx: usize,
}

pub struct PlayerStateStruct {
    pub channels: u16,
    pub sample_rate: u32,
    pub current_id: Option<Uuid>,
    pub waiting_for_header: bool,
}

#[derive(PartialEq)]
pub enum GetReturn {
    Ok,
    QueueEmpty,
}

pub type RodioSink = Arc<Mutex<rodio::Sink>>;
pub type ClientState = Arc<Mutex<ClientStateStruct>>;
pub type PlayerState = Arc<Mutex<PlayerStateStruct>>;

pub enum UiRequest {
    Prompt { s: String, prompt: String },
    Display(String),
    Shutdown,
}

pub struct UiResponse(pub String);

#[derive(Tabled)]
pub struct SongTable {
    #[tabled(rename = "Sl.no")]
    pub id: u16,

    #[tabled(rename = "*")]
    pub playing: PlayingDisplay,

    #[tabled(rename = "Title")]
    pub title: String,

    #[tabled(rename = "Artists")]
    pub artists: String,

    #[tabled(rename = "Length")]
    pub duration: String,
}

#[derive(Tabled)]
pub struct PlaylistTable {
    #[tabled(rename = "Sl.no")]
    pub id: u16,

    #[tabled(rename = "Title")]
    pub name: String,

    #[tabled(rename = "Entries")]
    pub length: usize,
}

pub struct PlayingDisplay(pub bool);

impl fmt::Display for PlayingDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.0 { "▶" } else { " " })
    }
}
