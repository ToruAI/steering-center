use axum::{
    extract::{Path, State},
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
        } else if !process.socket_path.is_empty()
            && PathBuf::from(&process.socket_path).exists()
        {
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
                .unwrap_or_else(|| "".to_string()),
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

    let plugin_statuses: Vec<PluginStatus> = plugins
        .values()
        .map(PluginStatus::from)
        .collect();

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

    supervisor
        .enable_plugin(&id)
        .await
        .map_err(|e| {
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

    supervisor
        .disable_plugin(&id)
        .await
        .map_err(|e| {
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

    let content = fs::read_to_string(&bundle_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        [(header::CONTENT_TYPE, "application/javascript")],
        content,
    ))
}

/// Get plugin logs
async fn get_plugin_logs(
    _auth: AdminUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    let log_path = format!("/var/log/toru/plugins/{}.log", id);

    let content = fs::read_to_string(&log_path).map_err(|_| StatusCode::NOT_FOUND)?;

    let logs: Vec<LogEntry> = content
        .lines()
        .filter_map(|line| {
            serde_json::from_str::<LogEntry>(line).ok()
        })
        .collect();

    Ok(Json(logs))
}

#[derive(Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
