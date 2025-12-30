pub mod error;
pub mod message;
pub mod protocol;
pub mod types;

pub use error::{PluginError, PluginResult};
pub use message::Message;
pub use protocol::PluginProtocol;
pub use types::*;

#[async_trait::async_trait]
pub trait ToruPlugin {
    fn metadata() -> PluginMetadata;

    async fn init(&mut self, ctx: PluginContext) -> PluginResult<()>;

    async fn handle_http(&self, req: HttpRequest) -> PluginResult<HttpResponse>;

    async fn handle_kv(&mut self, op: KvOp) -> PluginResult<Option<String>>;
}
