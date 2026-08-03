pub mod manifest;
pub mod movie;
pub mod series;
pub mod trigger;
pub mod play;
pub mod stream;

use crate::config::Config;
use crate::qbittorrent::QbitClient;
use axum::Router;
use axum::routing::get;
use reqwest::Client;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub qbit: Arc<QbitClient>,
    pub http_client: Client,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/manifest.json", get(manifest::manifest))
        .route("/stream/movie/{id}", get(movie::movie_stream))
        .route("/stream/series/{id}", get(series::series_stream))
        .route("/trigger-download/{stremio_id}/{info_hash}", get(trigger::trigger_download))
        .route("/play-file/{stremio_id}/{info_hash}", get(play::play_file))
        .route("/stream-file/{info_hash}", get(stream::stream_file))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
