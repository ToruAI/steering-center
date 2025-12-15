use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}

/// Stores the child process handle for cancellation
pub type TaskRegistry = Arc<Mutex<HashMap<String, Arc<Mutex<Option<tokio::process::Child>>>>>>;

pub fn create_task_registry() -> TaskRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Spawns a script and returns stdout/stderr handles separately.
/// The Child is wrapped for safe cancellation while streaming.
pub async fn execute_script(
    script_path: &str,
) -> Result<tokio::process::Child> {
    let child = TokioCommand::new("sh")
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    Ok(child)
}

/// Stores task handle in registry for cancellation support
pub async fn store_task(task_id: String, child: tokio::process::Child, registry: &TaskRegistry) {
    let mut reg = registry.lock().await;
    reg.insert(task_id, Arc::new(Mutex::new(Some(child))));
}

/// Gets the task handle from registry (does not remove it)
pub async fn get_task(task_id: &str, registry: &TaskRegistry) -> Option<Arc<Mutex<Option<tokio::process::Child>>>> {
    let reg = registry.lock().await;
    reg.get(task_id).cloned()
}

/// Removes task from registry (called after task completes)
pub async fn remove_task(task_id: &str, registry: &TaskRegistry) {
    let mut reg = registry.lock().await;
    reg.remove(task_id);
}

/// Cancels a running task by killing the child process
pub async fn cancel_task(task_id: &str, registry: &TaskRegistry) -> Result<bool> {
    let task_handle = {
        let reg = registry.lock().await;
        reg.get(task_id).cloned()
    };
    
    if let Some(handle) = task_handle {
        let mut child_opt = handle.lock().await;
        if let Some(ref mut child) = *child_opt {
            child.kill().await?;
            *child_opt = None; // Mark as killed
            return Ok(true);
        }
    }
    Ok(false)
}


