use crate::queue;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Track {
    pub name: String,
}

pub type Sender = queue::Sender<Track>;

pub type Receiver = queue::Receiver<Track>;

pub fn channel() -> (Sender, Receiver) {
    queue::channel::<Track>()
}
