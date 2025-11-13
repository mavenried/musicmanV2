use std::sync::Arc;
use uuid::Uuid;

pub struct State {
    pub songid: Option<Uuid>,
    pub queue: Vec<Uuid>,
}
pub type RodioSink = Arc<rodio::Sink>;

pub enum UiRequest {
    Prompt { s: String, prompt: String },
    Display(String),
    GetMeta(Vec<Uuid>),
}

pub struct UiResponse(pub String);
