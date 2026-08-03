use axum::{extract::{Path, Query, State, Request}, response::{IntoResponse, Response}};
use std::collections::HashMap;
use tower_http::services::ServeFile;
use tower::ServiceExt;
use super::AppState;

pub async fn stream_file(
    State(state): State<AppState>,
    Path(_info_hash): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    req: Request,
) -> Response {
    let file_path = params.get("filePath").unwrap_or(&String::new()).clone();
    let full_path = std::path::Path::new(&state.config.download_path).join(&file_path);
    
    if !full_path.starts_with(&state.config.download_path) {
        return axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Not Found"))
            .unwrap();
    }
    
    ServeFile::new(full_path).oneshot(req).await.unwrap().into_response()
}
