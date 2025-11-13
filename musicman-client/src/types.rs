use std::sync::Arc;
use uuid::Uuid;

pub struct State {
    pub songid: Option<Uuid>,
    pub queue: Vec<Uuid>,
}
pub type RodioSink = Arc<rodio::Sink>;
