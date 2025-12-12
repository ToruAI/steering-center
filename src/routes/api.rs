use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::{self, DbPool, QuickAction, TaskHistory};
use crate::services::system::{get_system_resources, SystemResources};
use sysinfo::System;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub sys: Arc<Mutex<System>>,
}

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/resources", get(resources))
        .route("/scripts", get(list_scripts))
        .route("/settings", get(get_settings))
        .route("/settings/:key", put(update_setting))
        .route("/history", get(get_history))
        .route("/quick-actions", get(get_quick_actions))
        .route("/quick-actions", post(create_quick_action))
        .route("/quick-actions/:id", delete(delete_quick_action))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn resources(State(state): State<AppState>) -> Result<Json<SystemResources>, StatusCode> {
    let mut sys = state.sys.lock().await;
    let resources = get_system_resources(&mut sys);
    Ok(Json(resources))
}

async fn list_scripts(State(state): State<AppState>) -> Result<Json<Vec<String>>, StatusCode> {
    let scripts_dir = db::get_setting(&state.db, "scripts_dir")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| "./scripts".to_string());
    
    let dir = PathBuf::from(&scripts_dir);
    let mut scripts = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".sh") || name.ends_with(".bash") {
                    scripts.push(name.to_string());
                }
            }
        }
    }
    
    Ok(Json(scripts))
}

#[derive(Serialize)]
struct SettingsResponse {
    settings: Vec<db::Setting>,
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<SettingsResponse>, StatusCode> {
    let settings = db::get_all_settings(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(SettingsResponse { settings }))
}

#[derive(Deserialize)]
struct UpdateSettingRequest {
    value: String,
}

async fn update_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSettingRequest>,
) -> Result<StatusCode, StatusCode> {
    db::set_setting(&state.db, &key, &payload.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_history(State(state): State<AppState>) -> Result<Json<Vec<TaskHistory>>, StatusCode> {
    let history = db::get_task_history(&state.db, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(history))
}

async fn get_quick_actions(
    State(state): State<AppState>,
) -> Result<Json<Vec<QuickAction>>, StatusCode> {
    let actions = db::get_quick_actions(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(actions))
}

#[derive(Deserialize)]
struct CreateQuickActionRequest {
    name: String,
    script_path: String,
    icon: Option<String>,
    display_order: Option<i32>,
}

async fn create_quick_action(
    State(state): State<AppState>,
    Json(payload): Json<CreateQuickActionRequest>,
) -> Result<Json<QuickAction>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let action = QuickAction {
        id,
        name: payload.name,
        script_path: payload.script_path,
        icon: payload.icon,
        display_order: payload.display_order.unwrap_or(0),
    };
    
    db::create_quick_action(&state.db, &action)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(action))
}

async fn delete_quick_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    db::delete_quick_action(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
