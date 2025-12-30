# Toru Plugin API

Rust SDK for creating plugins for Toru Steering Center.

## Overview

Toru plugins run as separate processes and communicate with the core system via Unix domain sockets. This provides:

- **Crash isolation** - Plugin failures don't crash the core
- **Full capabilities** - Plugins can execute shell commands, access files, network, database
- **Language flexibility** - Support for Rust (SDK) and Python (manual protocol implementation)

## Quick Start

### 1. Create a Plugin

```rust
use toru_plugin_api::{
    async_trait, HttpResponse, HttpRequest, KvOp, PluginConfig, PluginContext, PluginError, ToruPlugin,
};
use std::collections::HashMap;

struct MyPlugin {
    instance_id: String,
}

impl MyPlugin {
    fn new() -> Self {
        Self {
            instance_id: String::new(),
        }
    }
}

#[async_trait::async_trait]
impl ToruPlugin for MyPlugin {
    fn metadata() -> toru_plugin_api::PluginMetadata {
        toru_plugin_api::PluginMetadata {
            id: "my-plugin".to_string(),
            name: "My Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Your Name".to_string()),
            icon: "🔌".to_string(),
            route: "/my-plugin".to_string(),
        }
    }

    async fn init(&mut self, ctx: PluginContext) -> Result<(), PluginError> {
        self.instance_id = ctx.instance_id;
        println!("Plugin initialized for instance: {}", self.instance_id);

        // Store initial config
        ctx.kv.set("initialized", "true").await?;

        Ok(())
    }

    async fn handle_http(&self, req: HttpRequest) -> Result<HttpResponse, PluginError> {
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/my-plugin") => Ok(HttpResponse {
                status: 200,
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
                body: Some(r#"{"status":"ok","message":"Hello from plugin!"}"#.to_string()),
            }),
            ("POST", "/my-plugin/data") => {
                Ok(HttpResponse {
                    status: 200,
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("Content-Type".to_string(), "application/json".to_string());
                        h
                    },
                    body: Some(r#"{"status":"created"}"#.to_string()),
                })
            },
            _ => Ok(HttpResponse {
                status: 404,
                headers: HashMap::new(),
                body: Some(r#"{"error":"Not found"}"#.to_string()),
            }),
        }
    }

    async fn handle_kv(&mut self, op: KvOp) -> Result<Option<String>, PluginError> {
        match op {
            KvOp::Get { key } => {
                // Handle KV get
                Ok(None)
            },
            KvOp::Set { key, value } => {
                // Handle KV set
                Ok(None)
            },
            KvOp::Delete { key } => {
                // Handle KV delete
                Ok(None)
            },
        }
    }
}
```

### 2. Create the Binary Entrypoint

```rust
use std::env;
use toru_plugin_api::{PluginProtocol, ToruPlugin, Message, LifecycleInitPayload, PluginContext, PluginConfig, PluginKvStore};
use tokio::net::UnixListener;

struct MockKvStore;

#[async_trait::async_trait]
impl PluginKvStore for MockKvStore {
    async fn get(&self, key: &str) -> toru_plugin_api::PluginResult<Option<String>> {
        Ok(None)
    }

    async fn set(&self, key: &str, value: &str) -> toru_plugin_api::PluginResult<()> {
        Ok(())
    }

    async fn delete(&self, key: &str) -> toru_plugin_api::PluginResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle --metadata flag
    if args.len() > 1 && args[1] == "--metadata" {
        let metadata = MyPlugin::metadata();
        println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
        return;
    }

    let mut plugin = MyPlugin::new();
    let metadata = plugin.metadata();

    // Get socket path from environment or use default
    let socket_path = env::var("TORU_PLUGIN_SOCKET")
        .unwrap_or_else(|_| format!("/tmp/toru-plugins/{}.sock", metadata.id));

    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
    println!("Plugin listening on: {}", socket_path);

    let protocol = PluginProtocol::new();

    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut protocol_clone = protocol.clone();

                tokio::spawn(async move {
                    // Read init message
                    if let Ok(message) = protocol_clone.read_message(&mut stream).await {
                        // Handle message
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
```

### 3. Build and Package

```bash
# Build the plugin
cargo build --release

# Copy to plugins directory
cp target/release/my-plugin ../plugins/my-plugin.binary

# Make executable
chmod +x ../plugins/my-plugin.binary
```

## Plugin Metadata

Every plugin must provide metadata via the `--metadata` flag:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "icon": "🔌",
  "route": "/my-plugin"
}
```

## Message Protocol

Plugins communicate via JSON messages over Unix domain sockets:

### Lifecycle Message

```json
{
  "type": "lifecycle",
  "timestamp": "2025-12-30T12:00:00Z",
  "payload": {
    "action": "init",
    "instance_id": "uuid-here",
    "plugin_socket": "/tmp/toru-plugins/my-plugin.sock",
    "log_path": "/var/log/toru/plugins/my-plugin.log"
  }
}
```

### HTTP Message

```json
{
  "type": "http",
  "timestamp": "2025-12-30T12:00:00Z",
  "request_id": "uuid-here",
  "payload": {
    "method": "GET",
    "path": "/my-plugin",
    "headers": {},
    "body": null
  }
}
```

### KV Message

```json
{
  "type": "kv",
  "timestamp": "2025-12-30T12:00:00Z",
  "request_id": "uuid-here",
  "payload": {
    "action": "get",
    "key": "my-setting"
  }
}
```

## License

MIT
