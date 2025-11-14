use musicman_protocol::SongMeta;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::tcp::OwnedWriteHalf,
    sync::{Mutex, mpsc},
};
use uuid::Uuid;

pub struct State {
    pub current_stream_cancel: Option<mpsc::Sender<()>>,
}

pub type WriteSocket = Arc<Mutex<OwnedWriteHalf>>;
pub type SongIndex = HashMap<Uuid, SongMeta>;
