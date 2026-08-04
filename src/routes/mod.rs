pub mod admin;
pub mod manifest;
pub mod movie;
pub mod series;
pub mod trigger;
pub mod play;
pub mod stream;
pub mod home;
pub mod ui;

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
    pub db: Arc<crate::db::DbClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home::home))
        .route("/{api_key}/manifest.json", get(manifest::manifest))
        .route("/{api_key}/stream/movie/{id}", get(movie::movie_stream))
        .route("/{api_key}/stream/series/{id}", get(series::series_stream))
        .route("/{api_key}/trigger-download/{stremio_id}/{info_hash}", get(trigger::trigger_download))
        .route("/{api_key}/play-file/{stremio_id}/{info_hash}", get(play::play_file))
        .route("/{api_key}/stream-file/{info_hash}", get(stream::stream_file))
        .nest("/admin", admin::admin_routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
