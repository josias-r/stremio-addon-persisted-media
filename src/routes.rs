use crate::config::Config;
use crate::models::{Stream, StreamResponse};
use crate::qbittorrent::QbitClient;
use crate::jackett::{fetch_jackett_results, TorznabParams};
use crate::cinemeta::get_cinemeta_title;
use crate::parser::{is_definitely_not_full_season, is_definitely_wrong_episode};

use axum::{
    extract::{Path, Query, State, Request},
    response::{IntoResponse, Json, Response, Redirect},
    routing::get,
    Router,
};
use reqwest::Client;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeFile;
use strsim::levenshtein;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub qbit: Arc<QbitClient>,
    pub http_client: Client,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/stream/movie/{id}", get(movie_stream))
        .route("/stream/series/{id}", get(series_stream))
        .route("/trigger-download/{stremio_id}/{info_hash}", get(trigger_download))
        .route("/play-file/{stremio_id}/{info_hash}", get(play_file))
        .route("/stream-file/{info_hash}", get(stream_file))
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
    
    let mut expected_title = id.clone();
    if let Some(title) = get_cinemeta_title(&state.http_client, "movie", &id).await {
        expected_title = title.clone();
        if state.config.jackett_search_type == "text" {
            fetch_plans = vec![TorznabParams::MovieText { query: title }];
        }
    }

    let response = build_stream_response(&state, &id, fetch_plans, None, &expected_title).await;
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

    let mut expected_title = id_str.clone();
    if let Some(title) = get_cinemeta_title(&state.http_client, "series", &series_id).await {
        expected_title = format!("{} S{:02}E{:02}", title, season, episode);
        if state.config.jackett_search_type == "text" {
            fetch_plans = vec![
                TorznabParams::SeriesText { query: title.clone(), season, episode },
                TorznabParams::SeriesSeasonText { query: title, season },
            ];
        }
    }

    let filter_fn = Arc::new(move |title: &str| is_definitely_wrong_episode(title, season, episode));
    
    let response = build_stream_response(&state, &id_str, fetch_plans, Some(filter_fn), &expected_title).await;
    Json(response)
}

struct StreamItem {
    stream: Stream,
    category: u8,
    seeders: u32,
}

async fn build_stream_response(
    state: &AppState,
    stremio_id: &str,
    fetch_plans: Vec<TorznabParams>,
    filter_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    expected_title: &str,
) -> StreamResponse {
    let mut stream_items = Vec::new();

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

    let base_id = stremio_id.split(':').next().unwrap_or(stremio_id);
    let tagged_torrents = state.qbit.get_torrents_by_tag(base_id).await;
    let downloaded_hashes: HashSet<String> = tagged_torrents.iter().map(|t| t.hash.to_lowercase()).collect();
    
    let video_extensions = [".mp4", ".mkv", ".avi", ".webm", ".mov"];
    let expected_title_lower = expected_title.to_lowercase();
    
    let mut torrents_with_metadata = HashSet::new();

    for torrent in &tagged_torrents {
        let files = state.qbit.get_torrent_files(&torrent.hash).await;
        if !files.is_empty() {
            torrents_with_metadata.insert(torrent.hash.to_lowercase());
        }

        let mut valid_files = Vec::new();
        for (idx, file) in files.iter().enumerate() {
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
                valid_files.push((idx, file.clone()));
            }
        }
        
        if valid_files.is_empty() {
            continue;
        }

        let mut best_idx = 0;
        let mut best_score = (0, 0); // (token_matches, size)
        
        let expected_tokens: Vec<&str> = expected_title_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        for (i, (_, file)) in valid_files.iter().enumerate() {
            let filename_lower = file.name.to_lowercase();
            let filename_tokens: HashSet<&str> = filename_lower
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .collect();
                
            let mut score = 0;
            for token in &expected_tokens {
                if filename_tokens.contains(token) {
                    score += 1;
                }
            }
            
            let file_score = (score, file.size);
            if file_score > best_score {
                best_score = file_score;
                best_idx = i;
            }
        }
        
        for (i, (idx, file)) in valid_files.into_iter().enumerate() {
            let size_gb = file.size as f64 / 1024.0 / 1024.0 / 1024.0;
            let progress_pct = (file.progress * 100.0).round();
            let category = if i == best_idx { 1 } else { 2 };
            
            stream_items.push(StreamItem {
                stream: Stream {
                    name: Some(if category == 1 { "Local Stream (Best Match)".to_string() } else { "Local Stream".to_string() }),
                    title: Some(format!("{}\nProgress: {}% | 💾 {:.2} GB", file.name, progress_pct, size_gb)),
                    url: Some(format!("{}/play-file/{}/{}?fileIdx={}&filePath={}", state.config.public_url, urlencoding::encode(stremio_id), torrent.hash, idx, urlencoding::encode(&file.name))),
                    description: None,
                    yt_id: None,
                    info_hash: None,
                    file_idx: None,
                    external_url: None,
                },
                category,
                seeders: 0,
            });
        }
    }

    for r in results {
        let magnet_or_url = r.magnet_uri.clone().or_else(|| r.link.clone());
        if magnet_or_url.is_none() {
            continue;
        }
        let magnet_or_url = magnet_or_url.unwrap();
        
        let info_hash = r.info_hash.clone().unwrap_or_else(|| "unknown".to_string());
        
        if torrents_with_metadata.contains(&info_hash.to_lowercase()) {
            continue;
        }

        let is_downloading = downloaded_hashes.contains(&info_hash.to_lowercase());
        let prefix = if is_downloading { "[Downloading/Downloaded]\n" } else { "" };
        let size_gb = r.size as f64 / 1024.0 / 1024.0 / 1024.0;
        
        stream_items.push(StreamItem {
            stream: Stream {
                name: Some(format!("Qbit - {}", r.tracker)),
                title: Some(format!("{}{}\n👤 {} Seeders | 💾 {:.2} GB", prefix, r.title, r.seeders, size_gb)),
                url: Some(format!("{}/trigger-download/{}/{}?magnet={}", state.config.public_url, urlencoding::encode(stremio_id), info_hash, urlencoding::encode(&magnet_or_url))),
                description: None,
                yt_id: None,
                info_hash: None,
                file_idx: None,
                external_url: None,
            },
            category: 3,
            seeders: r.seeders,
        });
    }

    stream_items.sort_by(|a, b| {
        a.category.cmp(&b.category).then_with(|| b.seeders.cmp(&a.seeders))
    });

    StreamResponse { 
        streams: stream_items.into_iter().map(|item| item.stream).collect() 
    }
}

async fn trigger_download(
    State(state): State<AppState>,
    Path((stremio_id, info_hash)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    req: Request,
) -> Response {
    let base_id = stremio_id.split(':').next().unwrap_or(&stremio_id);
    if let Some(magnet) = params.get("magnet") {
        state.qbit.add_torrent(magnet, base_id).await;
    }
    
    let mut files = vec![];
    for _ in 0..30 {
        files = state.qbit.get_torrent_files(&info_hash).await;
        if !files.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    if files.is_empty() {
        let full_path = std::path::Path::new("placeholder.mp4");
        return ServeFile::new(full_path).oneshot(req).await.unwrap().into_response();
    }
    
    let video_extensions = [".mp4", ".mkv", ".avi", ".webm", ".mov"];
    let mut expected_title = stremio_id.clone();
    let is_series = stremio_id.contains(':');
    let mut season = 1;
    let mut episode = 1;
    
    if is_series {
        let parts: Vec<&str> = stremio_id.split(':').collect();
        let series_id = parts[0].to_string();
        season = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
        episode = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
        
        if let Some(t) = get_cinemeta_title(&state.http_client, "series", &series_id).await {
            expected_title = format!("{} S{:02}E{:02}", t, season, episode);
        }
    } else {
        if let Some(t) = get_cinemeta_title(&state.http_client, "movie", &stremio_id).await {
            expected_title = t;
        }
    }
    
    let expected_title_lower = expected_title.to_lowercase();
    let mut valid_files = Vec::new();
    
    for (idx, file) in files.iter().enumerate() {
        let ext = file.name.rsplit('.').next().unwrap_or("").to_lowercase();
        let mut is_video = false;
        for vext in &video_extensions {
            if vext.ends_with(&ext) || format!(".{}", ext) == *vext {
                is_video = true;
                break;
            }
        }
        
        if is_video {
            if is_series && is_definitely_wrong_episode(&file.name, season, episode) {
                continue;
            }
            valid_files.push((idx, file.clone()));
        }
    }
    
    if valid_files.is_empty() {
        for (idx, file) in files.iter().enumerate() {
            let ext = file.name.rsplit('.').next().unwrap_or("").to_lowercase();
            for vext in &video_extensions {
                if vext.ends_with(&ext) || format!(".{}", ext) == *vext {
                    valid_files.push((idx, file.clone()));
                    break;
                }
            }
        }
    }

    if valid_files.is_empty() {
        let full_path = std::path::Path::new("placeholder.mp4");
        return ServeFile::new(full_path).oneshot(req).await.unwrap().into_response();
    }
    
    let mut best_idx = 0;
    let mut best_score = (0, 0);
    
    let expected_tokens: Vec<&str> = expected_title_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    for (i, (_, file)) in valid_files.iter().enumerate() {
        let filename_lower = file.name.to_lowercase();
        let filename_tokens: HashSet<&str> = filename_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();
            
        let mut score = 0;
        for token in &expected_tokens {
            if filename_tokens.contains(token) {
                score += 1;
            }
        }
        
        let file_score = (score, file.size);
        if file_score > best_score {
            best_score = file_score;
            best_idx = i;
        }
    }
    
    let (chosen_idx, chosen_file) = valid_files[best_idx].clone();
    
    let all_ids: Vec<usize> = (0..files.len()).collect();
    state.qbit.set_file_priorities(&info_hash, &all_ids, 0).await;
    state.qbit.set_file_priorities(&info_hash, &[chosen_idx], 1).await;
    
    let redirect_url = format!("{}/stream-file/{}?filePath={}", state.config.public_url, info_hash, urlencoding::encode(&chosen_file.name));
    Redirect::temporary(&redirect_url).into_response()
}

async fn play_file(
    State(state): State<AppState>,
    Path((_stremio_id, info_hash)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Some(idx_str) = params.get("fileIdx") {
        if let Ok(idx) = idx_str.parse::<usize>() {
            state.qbit.set_file_priorities(&info_hash, &[idx], 1).await;
        }
    }
    
    let file_path = params.get("filePath").unwrap_or(&String::new()).clone();
    let redirect_url = format!("{}/stream-file/{}?filePath={}", state.config.public_url, info_hash, urlencoding::encode(&file_path));
    Redirect::temporary(&redirect_url).into_response()
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
