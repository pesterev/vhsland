use crate::{
    signal::shutdown_signal,
    track::{Sender, Track},
};
use bytes::{buf::BufExt, Bytes};
use hyper::{
    body,
    header::{HeaderValue, CONTENT_TYPE},
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{error, io, result, sync::Arc};
use tokio::{
    stream::StreamExt,
    sync::{watch::Receiver, Mutex},
};

#[derive(Clone)]
pub struct Context {
    pub chunk_recv: Receiver<Bytes>,
    pub track_sender: Sender,
    pub current_track: Arc<Mutex<Option<Track>>>,
}

impl Context {
    pub async fn spawn(self) {
        tokio::spawn(self.serve())
            .await
            .expect("start serving routes")
    }

    async fn serve(self) {
        let addr = ([0, 0, 0, 0], 3000).into();

        let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
            let context = self.clone();

            async { Ok::<_, Error>(service_fn(move |req| handle_req(context.clone(), req))) }
        }));

        info!("listening on http://{}", addr);

        let server = server.with_graceful_shutdown(shutdown_signal());

        if let Err(err) = server.await {
            error!("{}", err);
        }
    }
}

async fn handle_req(context: Context, req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => streaming(context.chunk_recv).await,
        (&Method::GET, "/tracks/current") => current_track(context.current_track).await,
        (&Method::POST, "/tracks/add") => add_track(req, context.track_sender).await,
        _ => not_found().await,
    }
}

async fn streaming(chunk_recv: Receiver<Bytes>) -> Result<Response<Body>> {
    let chunk_steam = chunk_recv.map(Ok::<_, io::Error>);
    let response = Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("audio/aac"))
        .body(Body::wrap_stream(chunk_steam))?;

    Ok(response)
}

async fn current_track(track: Arc<Mutex<Option<Track>>>) -> Result<Response<Body>> {
    let guard = track.lock().await;
    let track = match &*guard {
        Some(track) => track,
        None => return not_found().await,
    };
    let serialized = serde_json::to_string(track).expect("serialize current track");

    drop(guard);

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serialized))?;
    Ok(response)
}

#[derive(Deserialize, Serialize)]
enum Direction {
    Front,
    Back,
}

#[derive(Deserialize, Serialize)]
struct AddTrackBody {
    track: Track,
    direction: Direction,
}

async fn add_track(req: Request<Body>, track_sender: Sender) -> Result<Response<Body>> {
    let body_buf = body::aggregate(req.into_body()).await?;
    let deserialized: AddTrackBody = match serde_json::from_reader(body_buf.reader()) {
        Ok(track) => track,
        Err(_) => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())?;
            return Ok(response);
        }
    };
    match deserialized.direction {
        Direction::Front => track_sender.push_front(deserialized.track),
        Direction::Back => track_sender.push_back(deserialized.track),
    };
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())?;
    Ok(response)
}

async fn not_found() -> Result<Response<Body>> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?;
    Ok(response)
}

pub type Error = Box<dyn error::Error + Send + Sync>;

pub type Result<T> = result::Result<T, Error>;
