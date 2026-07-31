mod config;
mod models;
mod qbittorrent;
mod cinemeta;
mod jackett;
mod parser;
mod routes;

use std::sync::Arc;
use reqwest::Client;
use log::info;

#[tokio::main]
async fn main() {
    // Initialize env_logger. Will read RUST_LOG env var or default to info
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info,mini_media_server_addon=debug");
        }
    }
    env_logger::init();
    
    let config = config::Config::load();
    let qbit = qbittorrent::QbitClient::new(config.clone());
    
    let state = routes::AppState {
        config: Arc::new(config.clone()),
        qbit: Arc::new(qbit),
        http_client: Client::builder().build().unwrap(),
    };
    
    let app = routes::create_router(state);
    
    let addr = format!("0.0.0.0:{}", config.port);
    info!("Modular Stremio Add-on server is running at http://localhost:{}", config.port);
    info!("Manifest URL: http://localhost:{}/manifest.json", config.port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
