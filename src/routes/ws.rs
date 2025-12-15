use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::db;
use crate::routes::api::AppState;
use crate::services::executor::{self, TaskMessage, TaskRegistry};

#[derive(Deserialize)]
struct ClientMessage {
    r#type: String,
    script: Option<String>,
    task_id: Option<String>,
}

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let registry = executor::create_task_registry();
    
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };
        
        let text = match msg.to_text() {
            Ok(text) => text,
            Err(_) => continue,
        };
        
        let client_msg: ClientMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(_) => continue,
        };
        
        match client_msg.r#type.as_str() {
            "run" => {
                if let Some(script_name) = client_msg.script {
                    let scripts_dir = db::get_setting(&state.db, "scripts_dir")
                        .await
                        .unwrap_or_else(|_| Some("./scripts".to_string()))
                        .unwrap_or_else(|| "./scripts".to_string());
                    
                    let script_path = format!("{}/{}", scripts_dir, script_name);
                    let task_id = Uuid::new_v4().to_string();
                    
                    // Create task history entry
                    let task_history = crate::db::TaskHistory {
                        id: task_id.clone(),
                        script_name: script_name.clone(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        finished_at: None,
                        exit_code: None,
                        output: None,
                    };
                    
                    if let Err(e) = db::insert_task_history(&state.db, &task_history).await {
                        eprintln!("Failed to insert task history: {}", e);
                    }
                    
                    // Send started message
                    let started_msg = TaskMessage {
                        r#type: "started".to_string(),
                        task_id: Some(task_id.clone()),
                        data: None,
                        code: None,
                    };
                    
                    {
                        let mut s = sender.lock().await;
                        if s.send(Message::Text(
                            serde_json::to_string(&started_msg).unwrap(),
                        )).await.is_err() {
                            break;
                        }
                    }
                    
                    // Execute script
                    let mut child = match executor::execute_script(&script_path).await {
                        Ok(child) => child,
                        Err(e) => {
                            let error_msg = TaskMessage {
                                r#type: "error".to_string(),
                                task_id: Some(task_id.clone()),
                                data: Some(format!("Failed to start script: {}", e)),
                                code: None,
                            };
                            let mut s = sender.lock().await;
                            let _ = s.send(Message::Text(
                                serde_json::to_string(&error_msg).unwrap(),
                            )).await;
                            continue;
                        }
                    };
                    
                    // Take stdout/stderr BEFORE storing in registry
                    let stdout = child.stdout.take().expect("stdout not captured");
                    let stderr = child.stderr.take().expect("stderr not captured");
                    
                    // Store in registry for cancellation (child still owns the process handle)
                    executor::store_task(task_id.clone(), child, &registry).await;
                    
                    // Spawn streaming task
                    let task_id_clone = task_id.clone();
                    let sender_clone = sender.clone();
                    let db_clone = state.db.clone();
                    let registry_clone = registry.clone();
                    
                    tokio::spawn(async move {
                        stream_task_output(
                            task_id_clone,
                            stdout,
                            stderr,
                            sender_clone,
                            db_clone,
                            registry_clone,
                        ).await;
                    });
                }
            }
            "cancel" => {
                if let Some(task_id) = client_msg.task_id {
                    if executor::cancel_task(&task_id, &registry).await.unwrap_or(false) {
                        let cancelled_msg = TaskMessage {
                            r#type: "cancelled".to_string(),
                            task_id: Some(task_id.clone()),
                            data: None,
                            code: None,
                        };
                        let mut s = sender.lock().await;
                        let _ = s.send(Message::Text(
                            serde_json::to_string(&cancelled_msg).unwrap(),
                        )).await;
                        
                        // Clean up registry
                        executor::remove_task(&task_id, &registry).await;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Streams stdout/stderr from a running task to the WebSocket client
async fn stream_task_output(
    task_id: String,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    sender: Arc<Mutex<futures::stream::SplitSink<axum::extract::ws::WebSocket, Message>>>,
    db: crate::db::DbPool,
    registry: TaskRegistry,
) {
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);
    let mut output = String::new();
    let mut stdout_line = String::new();
    let mut stderr_line = String::new();
    let mut stdout_done = false;
    let mut stderr_done = false;
    
    // Stream output until both streams are closed
    while !stdout_done || !stderr_done {
        tokio::select! {
            result = stdout_reader.read_line(&mut stdout_line), if !stdout_done => {
                match result {
                    Ok(0) => stdout_done = true,
                    Ok(_) => {
                        let line = stdout_line.clone();
                        output.push_str(&line);
                        let msg = TaskMessage {
                            r#type: "stdout".to_string(),
                            task_id: Some(task_id.clone()),
                            data: Some(line.trim_end().to_string()),
                            code: None,
                        };
                        let mut s = sender.lock().await;
                        let _ = s.send(Message::Text(
                            serde_json::to_string(&msg).unwrap(),
                        )).await;
                        stdout_line.clear();
                    }
                    Err(_) => stdout_done = true,
                }
            }
            result = stderr_reader.read_line(&mut stderr_line), if !stderr_done => {
                match result {
                    Ok(0) => stderr_done = true,
                    Ok(_) => {
                        let line = stderr_line.clone();
                        output.push_str(&line);
                        let msg = TaskMessage {
                            r#type: "stderr".to_string(),
                            task_id: Some(task_id.clone()),
                            data: Some(line.trim_end().to_string()),
                            code: None,
                        };
                        let mut s = sender.lock().await;
                        let _ = s.send(Message::Text(
                            serde_json::to_string(&msg).unwrap(),
                        )).await;
                        stderr_line.clear();
                    }
                    Err(_) => stderr_done = true,
                }
            }
        }
    }
    
    // Get the child process to wait for exit
    let exit_code = if let Some(handle) = executor::get_task(&task_id, &registry).await {
        let mut child_opt = handle.lock().await;
        if let Some(ref mut child) = *child_opt {
            let status = child.wait().await;
            status.ok().and_then(|s| s.code()).unwrap_or(-1)
        } else {
            // Process was killed (cancelled)
            -1
        }
    } else {
        -1
    };
    
    // Clean up registry
    executor::remove_task(&task_id, &registry).await;
    
    // Update task history
    let finished_at = chrono::Utc::now().to_rfc3339();
    let output_str = if output.is_empty() { None } else { Some(output.as_str()) };
    let _ = db::update_task_history(
        &db,
        &task_id,
        &finished_at,
        exit_code,
        output_str,
    ).await;
    
    // Send exit message
    let exit_msg = TaskMessage {
        r#type: "exit".to_string(),
        task_id: Some(task_id),
        data: None,
        code: Some(exit_code),
    };
    let mut s = sender.lock().await;
    let _ = s.send(Message::Text(
        serde_json::to_string(&exit_msg).unwrap(),
    )).await;
}
