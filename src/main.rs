mod api;
mod codec;
mod core;
mod queue;
mod signal;
mod track;

use bytes::Bytes;
use futures::join;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let media_dir = dotenv::var("MEDIA_DIR").expect("get media directory from env vars");

    let (track_sender, track_recv) = track::channel();
    let (chunk_sender, chunk_recv) = watch::channel::<Bytes>(Bytes::new());
    let current_track = Arc::new(Mutex::new(None));

    let core_context = core::Context {
        media_dir,
        track_recv,
        chunk_sender,
        current_track: Arc::clone(&current_track),
    };

    let api_context = api::Context {
        chunk_recv,
        track_sender,
        current_track,
    };

    join!(core_context.spawn(), api_context.spawn());
}
