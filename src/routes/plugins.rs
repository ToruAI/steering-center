use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::routes::api::AppState;
use crate::routes::auth::AdminUser;
use crate::services::logging::LogLevel;
use crate::services::plugins::PluginProcess;

/// Plugin status information
#[derive(Serialize, Clone)]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub icon: String,
    pub enabled: bool,
    pub running: bool,
    pub health: String, // "healthy", "unhealthy", "disabled"
    pub pid: Option<u32>,
    pub socket_path: Option<String>,
}

impl From<&PluginProcess> for PluginStatus {
    fn from(process: &PluginProcess) -> Self {
        let health = if !process.enabled {
            "disabled".to_string()
        } else if !process.socket_path.is_empty() && PathBuf::from(&process.socket_path).exists() {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        };

        PluginStatus {
            id: process.id.clone(),
            name: process
                .metadata
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or_else(|| process.id.clone()),
            version: process
                .metadata
                .as_ref()
                .map(|m| m.version.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            author: process.metadata.as_ref().and_then(|m| m.author.clone()),
            icon: process
                .metadata
                .as_ref()
                .map(|m| m.icon.clone())
                .unwrap_or_default(),
            enabled: process.enabled,
            running: process.process.is_some(),
            health,
            pid: process.pid,
            socket_path: if process.socket_path.is_empty() {
                None
            } else {
                Some(process.socket_path.clone())
            },
        }
    }
}

pub fn create_plugin_router() -> Router<AppState> {
    Router::new()
        // Admin-only routes
        .route("/", get(list_plugins))
        .route("/:id", get(get_plugin))
        .route("/:id/enable", post(enable_plugin))
        .route("/:id/disable", post(disable_plugin))
        .route("/:id/bundle.js", get(get_plugin_bundle))
        .route("/:id/logs", get(get_plugin_logs))
}

/// List all plugins
async fn list_plugins(
    _auth: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PluginStatus>>, StatusCode> {
    let supervisor = state
        .supervisor
        .as_ref()
        .ok_or(StatusCode::NOT_IMPLEMENTED)?
        .lock()
        .await;
    let plugins = supervisor.get_all_plugins();

    let plugin_statuses: Vec<PluginStatus> = plugins.values().map(PluginStatus::from).collect();

    Ok(Json(plugin_statuses))
}

/// Get plugin details
async fn get_plugin(
    _auth: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PluginStatus>, StatusCode> {
    let supervisor = state
        .supervisor
        .as_ref()
        .ok_or(StatusCode::NOT_IMPLEMENTED)?
        .lock()
        .await;
    let plugin = supervisor
        .get_plugin_status(&id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(PluginStatus::from(plugin)))
}

/// Enable a plugin
async fn enable_plugin(
    _auth: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut supervisor = state
        .supervisor
        .as_ref()
        .ok_or((
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({ "error": "Plugin supervisor not initialized" })),
        ))?
        .lock()
        .await;

    // Check if plugin exists
    if supervisor.get_plugin_status(&id).is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Plugin not found" })),
        ));
    }

    supervisor.enable_plugin(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to enable plugin: {}", e) })),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Disable a plugin
async fn disable_plugin(
    _auth: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut supervisor = state
        .supervisor
        .as_ref()
        .ok_or((
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({ "error": "Plugin supervisor not initialized" })),
        ))?
        .lock()
        .await;

    // Check if plugin exists
    if supervisor.get_plugin_status(&id).is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Plugin not found" })),
        ));
    }

    supervisor.disable_plugin(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to disable plugin: {}", e) })),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get plugin frontend bundle
async fn get_plugin_bundle(
    _auth: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let supervisor = state
        .supervisor
        .as_ref()
        .ok_or(StatusCode::NOT_IMPLEMENTED)?
        .lock()
        .await;
    let plugin = supervisor
        .get_plugin_status(&id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if plugin is enabled
    if !plugin.enabled {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get plugin bundle path from plugins directory
    let plugins_dir = supervisor.get_plugins_dir();
    let bundle_path = plugins_dir.join(&id).join("bundle.js");

    if !bundle_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content =
        fs::read_to_string(&bundle_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(([(header::CONTENT_TYPE, "application/javascript")], content))
}

#[derive(Deserialize)]
struct LogQuery {
    #[serde(default)]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
    #[serde(default)]
    level: Option<String>,
}

fn default_page_size() -> usize {
    100
}

/// Get plugin logs with pagination and filtering
async fn get_plugin_logs(
    _auth: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogsResponse>, StatusCode> {
    let supervisor = state
        .supervisor
        .as_ref()
        .ok_or(StatusCode::NOT_IMPLEMENTED)?
        .lock()
        .await;

    // Check if plugin exists
    if supervisor.get_plugin_status(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let plugin_logger = supervisor.plugin_logger();

    // Parse log level filter
    let filter_level = query.level.as_ref().and_then(|l| LogLevel::from_str(l));

    // Read logs with pagination and filtering
    let logs = plugin_logger
        .read_plugin_logs(&id, filter_level, query.page, query.page_size)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LogsResponse {
        logs,
        page: query.page,
        page_size: query.page_size,
    }))
}

#[derive(Serialize)]
struct LogsResponse {
    logs: Vec<crate::services::logging::LogEntry>,
    page: usize,
    page_size: usize,
}
