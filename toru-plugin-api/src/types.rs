use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub icon: String,
    pub route: String,
}

pub struct PluginContext {
    pub instance_id: String,
    pub config: PluginConfig,
    pub kv: Box<dyn PluginKvStore>,
}

#[derive(Debug, Clone, Default)]
pub struct PluginConfig {
    pub env: std::collections::HashMap<String, String>,
}

#[async_trait::async_trait]
pub trait PluginKvStore: Send + Sync {
    async fn get(&self, key: &str) -> crate::PluginResult<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> crate::PluginResult<()>;
    async fn delete(&self, key: &str) -> crate::PluginResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum KvOp {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleInitPayload {
    pub instance_id: String,
    pub plugin_socket: String,
    pub log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    #[serde(rename = "lifecycle")]
    Lifecycle {
        action: String,
        #[serde(flatten)]
        payload: Option<LifecycleInitPayload>,
    },
    #[serde(rename = "http")]
    Http {
        request_id: String,
        payload: HttpRequest,
    },
    #[serde(rename = "kv")]
    Kv {
        request_id: String,
        #[serde(flatten)]
        payload: KvMessagePayload,
    },
}

/// KV message payload - can be either a request (operation) or response (value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KvMessagePayload {
    Request(KvOp),
    Response { value: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "type")]
    pub message_type: String,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
    pub payload: MessagePayload,
}

impl Message {
    pub fn new_lifecycle(action: &str, init_payload: Option<LifecycleInitPayload>) -> Self {
        Self {
            message_type: "lifecycle".to_string(),
            timestamp: Utc::now(),
            request_id: None,
            payload: MessagePayload::Lifecycle {
                action: action.to_string(),
                payload: init_payload,
            },
        }
    }

    pub fn new_http(request_id: String, payload: HttpRequest) -> Self {
        let request_id_clone = request_id.clone();
        Self {
            message_type: "http".to_string(),
            timestamp: Utc::now(),
            request_id: Some(request_id),
            payload: MessagePayload::Http {
                request_id: request_id_clone,
                payload,
            },
        }
    }

    pub fn new_kv(request_id: String, payload: KvOp) -> Self {
        let request_id_clone = request_id.clone();
        Self {
            message_type: "kv".to_string(),
            timestamp: Utc::now(),
            request_id: Some(request_id),
            payload: MessagePayload::Kv {
                request_id: request_id_clone,
                payload: KvMessagePayload::Request(payload),
            },
        }
    }

    /// Create a KV response message (used by plugins to respond to KV operations)
    pub fn new_kv_response(request_id: String, value: Option<String>) -> Self {
        let request_id_clone = request_id.clone();
        Self {
            message_type: "kv".to_string(),
            timestamp: Utc::now(),
            request_id: Some(request_id),
            payload: MessagePayload::Kv {
                request_id: request_id_clone,
                payload: KvMessagePayload::Response { value },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMessageResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvMessageResponse {
    pub value: Option<String>,
}
