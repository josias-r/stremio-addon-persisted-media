use crate::config::Config;
use crate::models::{Stream, StreamResponse};
use crate::qbittorrent::QbitClient;
use crate::jackett::{fetch_jackett_results, TorznabParams};
use crate::cinemeta::get_cinemeta_title;
use crate::parser::{is_definitely_not_full_season, is_definitely_wrong_episode};

use axum::{
    extract::{Path, Query, State, Request},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeFile;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub qbit: Arc<QbitClient>,
    pub http_client: Client,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/stream/movie/:id", get(movie_stream))
        .route("/stream/series/:id", get(series_stream))
        .route("/trigger-download/:stremio_id/:info_hash", get(trigger_download))
        .route("/stream-file/:info_hash", get(stream_file))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn manifest() -> impl IntoResponse {
    Json(json!({
        "id": "mini-media-server.addon",
        "version": "1.0.0",
        "name": "Mini Media Server",
        "description": "A tiny Stremio addon acting similar to a self-hostable debrid service. Search for torrents via Jackett, download and cache them automatically and serve them to Stremio.",
        "resources": ["stream"],
        "types": ["movie", "series"],
        "catalogs": []
    }))
}

async fn movie_stream(
    State(state): State<AppState>,
    Path(id_ext): Path<String>,
) -> impl IntoResponse {
    let id = id_ext.replace(".json", "");
    
    let mut fetch_plans = vec![TorznabParams::MovieImdb { imdb_id: id.clone() }];
    if state.config.jackett_search_type == "text" {
        if let Some(title) = get_cinemeta_title(&state.http_client, "movie", &id).await {
            fetch_plans = vec![TorznabParams::MovieText { query: title }];
        }
    }

    let response = build_stream_response(&state, &id, fetch_plans, None).await;
    Json(response)
}

async fn series_stream(
    State(state): State<AppState>,
    Path(id_ext): Path<String>,
) -> impl IntoResponse {
    let id_str = id_ext.replace(".json", "");
    let parts: Vec<&str> = id_str.split(':').collect();
    let series_id = parts[0].to_string();
    let season: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let episode: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut fetch_plans = vec![
        TorznabParams::SeriesImdb { imdb_id: series_id.clone(), season, episode },
        TorznabParams::SeriesSeasonImdb { imdb_id: series_id.clone(), season },
    ];

    if state.config.jackett_search_type == "text" {
        if let Some(title) = get_cinemeta_title(&state.http_client, "series", &series_id).await {
            fetch_plans = vec![
                TorznabParams::SeriesText { query: title.clone(), season, episode },
                TorznabParams::SeriesSeasonText { query: title, season },
            ];
        }
    }

    let filter_fn = Arc::new(move |title: &str| is_definitely_wrong_episode(title, season, episode));
    
    let response = build_stream_response(&state, &id_str, fetch_plans, Some(filter_fn)).await;
    Json(response)
}

async fn build_stream_response(
    state: &AppState,
    stremio_id: &str,
    fetch_plans: Vec<TorznabParams>,
    filter_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
) -> StreamResponse {
    let mut streams = Vec::new();

    // Deduplicate Jackett results
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    for plan in fetch_plans {
        let plan_results = fetch_jackett_results(&state.http_client, &state.config, &plan).await;
        for r in plan_results {
            let key = r.info_hash.clone().unwrap_or_else(|| r.magnet_uri.clone().unwrap_or_else(|| r.title.clone())).to_lowercase();
            if !seen.contains(&key) {
                // Apply jackett filter if applicable
                let include = match (&plan, &filter_fn) {
                    (TorznabParams::SeriesSeasonImdb { season, .. }, _) | (TorznabParams::SeriesSeasonText { season, .. }, _) => {
                        !is_definitely_not_full_season(&r.title, *season)
                    }
                    (_, Some(f)) => !f(&r.title),
                    _ => true,
                };
                
                if include {
                    seen.insert(key);
                    results.push(r);
                }
            }
        }
    }

    // Get torrents tagged with this stremio_id
    let mut tagged_torrents = state.qbit.get_torrents_by_tag(stremio_id).await;
    
    // Automatically link Jackett results that are already in qBittorrent but not tagged yet (legacy support)
    // Actually, in the Rust rewrite we only rely on tags, but to be robust, if a Jackett hash is in qbit, it will be added.
    let downloaded_hashes: HashSet<String> = tagged_torrents.iter().map(|t| t.hash.to_lowercase()).collect();

    // Map local files
    let video_extensions = [".mp4", ".mkv", ".avi", ".webm", ".mov"];
    for torrent in &tagged_torrents {
        let files = state.qbit.get_torrent_files(&torrent.hash).await;
        for file in files {
            let ext = file.name.rsplit('.').next().unwrap_or("").to_lowercase();
            let mut is_video = false;
            for vext in &video_extensions {
                if vext.ends_with(&ext) || format!(".{}", ext) == *vext {
                    is_video = true;
                    break;
                }
            }

            if is_video {
                if let Some(f) = &filter_fn {
                    if f(&file.name) {
                        continue;
                    }
                }

                let size_gb = file.size as f64 / 1024.0 / 1024.0 / 1024.0;
                let progress_pct = (file.progress * 100.0).round();
                
                streams.push(Stream {
                    name: Some("Local Stream".to_string()),
                    title: Some(format!("{}\nProgress: {}% | 💾 {:.2} GB", file.name, progress_pct, size_gb)),
                    url: Some(format!("{}/stream-file/{}?filePath={}", state.config.public_url, torrent.hash, urlencoding::encode(&file.name))),
                    description: None,
                    yt_id: None,
                    info_hash: None,
                    file_idx: None,
                    external_url: None,
                });
            }
        }
    }

    // Map Jackett streams
    for r in results {
        let magnet_or_url = r.magnet_uri.clone().or_else(|| r.link.clone());
        if magnet_or_url.is_none() {
            continue;
        }
        let magnet_or_url = magnet_or_url.unwrap();
        
        let info_hash = r.info_hash.clone();
        if info_hash.is_none() {
            continue;
        }
        let info_hash = info_hash.unwrap();
        let is_downloading = downloaded_hashes.contains(&info_hash.to_lowercase());
        
        let prefix = if is_downloading { "[Downloading/Downloaded]\n" } else { "" };
        let size_gb = r.size as f64 / 1024.0 / 1024.0 / 1024.0;
        
        streams.push(Stream {
            name: Some(format!("Qbit - {}", r.tracker)),
            title: Some(format!("{}{}\n👤 {} Seeders | 💾 {:.2} GB", prefix, r.title, r.seeders, size_gb)),
            url: Some(format!("{}/trigger-download/{}/{}?magnet={}", state.config.public_url, urlencoding::encode(stremio_id), info_hash, urlencoding::encode(&magnet_or_url))),
            description: None,
            yt_id: None,
            info_hash: None,
            file_idx: None,
            external_url: None,
        });
    }

    StreamResponse { streams }
}

async fn trigger_download(
    State(state): State<AppState>,
    Path((stremio_id, _info_hash)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    req: Request,
) -> Response {
    if let Some(magnet) = params.get("magnet") {
        state.qbit.add_torrent(magnet, &stremio_id).await;
    }
    
    // Serve a tiny dummy mp4 or empty 200 response to satisfy the player temporarily
    let full_path = std::path::Path::new("placeholder.mp4");
    ServeFile::new(full_path).oneshot(req).await.unwrap().into_response()
}

async fn stream_file(
    State(state): State<AppState>,
    Path(_info_hash): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    req: Request,
) -> Response {
    let file_path = params.get("filePath").unwrap_or(&String::new()).clone();
    let full_path = std::path::Path::new(&state.config.download_path).join(&file_path);
    
    // Check for directory traversal
    if !full_path.starts_with(&state.config.download_path) {
        return axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Not Found"))
            .unwrap();
    }
    
    ServeFile::new(full_path).oneshot(req).await.unwrap().into_response()
}
