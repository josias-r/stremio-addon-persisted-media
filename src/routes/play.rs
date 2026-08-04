use axum::{extract::{Path, Query, State}, response::{IntoResponse, Response, Redirect}};
use std::collections::HashMap;
use super::AppState;

pub async fn play_file(
    State(state): State<AppState>,
    Path((_stremio_id, info_hash)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Some(idx_str) = params.get("fileIdx") {
        if let Ok(idx) = idx_str.parse::<usize>() {
            state.qbit.set_file_priorities(&info_hash, &[idx], 1).await;
            state.qbit.resume_torrent(&info_hash).await;
        }
    }
    
    let file_path = params.get("filePath").unwrap_or(&String::new()).clone();
    let redirect_url = format!("{}/stream-file/{}?filePath={}", state.config.public_url, info_hash, urlencoding::encode(&file_path));
    Redirect::temporary(&redirect_url).into_response()
}
