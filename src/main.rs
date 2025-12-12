mod db;
mod routes;
mod services;

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};

use crate::db::init_db;
use crate::routes::{create_api_router, handle_websocket};
use crate::routes::api::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // Initialize database
    let db = init_db()?;
    tracing::info!("Database initialized");
    
    // Initialize system monitor
    let sys = Arc::new(Mutex::new(System::new_all()));
    
    // Create app state
    let state = AppState { db, sys };
    
    // Create API router
    let api_router = create_api_router();
    
    // SPA fallback handler - serves index.html for non-API routes
    async fn spa_fallback() -> impl IntoResponse {
        match tokio::fs::read_to_string("frontend/dist/index.html").await {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Body::from(html))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Frontend not built. Run 'cd frontend && npm run build'"))
                .unwrap(),
        }
    }

    // Create main router
    let app = Router::new()
        .route("/api/ws", get(handle_websocket))
        .nest("/api", api_router)
        .nest_service("/", ServeDir::new("frontend/dist"))
        .fallback(spa_fallback)
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
