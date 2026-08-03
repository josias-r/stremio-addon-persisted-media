use axum::{extract::{Path, Query, State, Request}, response::{IntoResponse, Response}};
use std::collections::HashMap;
use tower_http::services::ServeFile;
use tower::ServiceExt;
use std::process::Stdio;
use tokio::process::Command;
use tokio_util::io::ReaderStream;
use axum::body::Body;
use serde_json::Value;
use super::AppState;

async fn is_simple_format(path: &std::path::Path) -> bool {
    let mut cmd = Command::new("ffprobe");
    cmd.arg("-v")
       .arg("error")
       .arg("-show_entries")
       .arg("stream=codec_name,codec_type")
       .arg("-of")
       .arg("json")
       .arg(path);

    if let Ok(output) = cmd.output().await {
        if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
            if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
                let mut has_video = false;
                let mut is_compatible_video = false;
                
                for stream in streams {
                    if let Some(codec_type) = stream.get("codec_type").and_then(|s| s.as_str()) {
                        if codec_type == "video" {
                            has_video = true;
                            if let Some(codec_name) = stream.get("codec_name").and_then(|s| s.as_str()) {
                                // Both H.264 and HEVC (H.265) can be safely remuxed to MP4
                                if codec_name == "h264" || codec_name == "hevc" {
                                    is_compatible_video = true;
                                }
                            }
                        }
                    }
                }
                // If it has video, it must be h264 or hevc to be considered simple.
                return has_video && is_compatible_video;
            }
        }
    }
    false
}

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
            .body(Body::from("Not Found"))
            .unwrap();
    }
    
    let ext = full_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    // If it's not natively supported by browsers, check if it contains a simple format
    if ext != "mp4" && ext != "webm" {
        if is_simple_format(&full_path).await {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-i")
                .arg(&full_path)
                .arg("-c")
                .arg("copy")
                .arg("-movflags")
                .arg("frag_keyframe+empty_moov+faststart")
                .arg("-f")
                .arg("mp4")
                .arg("pipe:1")
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let stream = ReaderStream::new(stdout);
                        let body = Body::from_stream(stream);

                        // Spawn a task to wait for the child process so it doesn't become a zombie
                        tokio::spawn(async move {
                            let _ = child.wait().await;
                        });

                        return axum::response::Response::builder()
                            .header("Content-Type", "video/mp4")
                            .body(body)
                            .unwrap();
                    }
                }
                Err(e) => {
                    log::error!("Failed to spawn ffmpeg: {}", e);
                }
            }
        }
    }
    
    // Fallback: serve the file directly.
    // ServeFile automatically adds the correct Content-Type header based on the file extension.
    ServeFile::new(full_path).oneshot(req).await.unwrap().into_response()
}
