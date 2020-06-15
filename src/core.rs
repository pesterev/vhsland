use crate::{
    codec::ChunksCodec,
    track::{Receiver, Track},
};
use bytes::Bytes;
use log::{error, info};
use mp3_metadata::read_from_file;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    fs::OpenOptions,
    stream::StreamExt,
    sync::{watch::Sender, Mutex},
    time::delay_for,
};
use tokio_util::codec::Framed;

pub struct Context {
    pub media_dir: String,
    pub track_recv: Receiver,
    pub chunk_sender: Sender<Bytes>,
    pub current_track: Arc<Mutex<Option<Track>>>,
}

impl Context {
    pub async fn spawn(self) {
        tokio::spawn(self.serve())
            .await
            .expect("spawn a streaming task");
    }

    async fn serve(mut self) {
        info!("start streaming a playlist");

        while let Some(track) = self.track_recv.next().await {
            let mut guard = self.current_track.lock().await;
            guard.replace(track.clone());
            drop(guard);
            if let Err(err) = self.handle_track(track.clone()).await {
                error!("failed to handle track '{}' : {}", track.name, err);
            }
        }
    }

    async fn handle_track(&mut self, track: Track) -> Result<(), Box<dyn std::error::Error>> {
        let path: PathBuf = [self.media_dir.as_str(), track.name.as_str()]
            .iter()
            .collect();

        let file = OpenOptions::new().read(true).open(path.clone()).await?;
        let file_metadata = file.metadata().await?;
        let bytes_len = file_metadata.len() as usize;

        let mp3_metadata = read_from_file(path.clone())?;
        let duration = mp3_metadata.duration.as_secs() as usize;
        let bytes_per_second = bytes_len / duration;

        let mut framed = Framed::new(file, ChunksCodec::new(bytes_per_second, duration));

        while let (Some(result), _) = (framed.next().await, delay_for(Duration::from_secs(1)).await)
        {
            let bytes = result?.freeze();
            self.chunk_sender.broadcast(bytes).unwrap();
        }

        Ok(())
    }
}
