use axum::response::{IntoResponse, Json};
use serde_json::json;

pub async fn manifest() -> impl IntoResponse {
    Json(json!({
        "id": "mini-media-server.addon",
        "version": "1.0.0",
        "name": "Mini Media Server",
        "description": "A tiny Stremio addon acting similar to a self-hostable debrid service.",
        "resources": ["stream"],
        "types": ["movie", "series"],
        "catalogs": []
    }))
}
