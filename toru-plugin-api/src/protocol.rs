use crate::{error::PluginResult, types::Message};
use tokio::net::UnixStream;

pub struct PluginProtocol;

impl PluginProtocol {
    pub fn new() -> Self {
        Self
    }

    pub async fn read_message(&mut self, stream: &mut UnixStream) -> PluginResult<Message> {
        use tokio::io::{AsyncReadExt, BufReader};

        let mut reader = BufReader::new(stream);
        let mut length_buf = [0u8; 4];

        reader.read_exact(&mut length_buf).await?;

        let length = u32::from_be_bytes(length_buf) as usize;
        let mut msg_buf = vec![0u8; length];

        reader.read_exact(&mut msg_buf).await?;

        let message: Message = serde_json::from_slice(&msg_buf)?;

        Ok(message)
    }

    pub async fn write_message(
        &self,
        stream: &mut UnixStream,
        message: &Message,
    ) -> PluginResult<()> {
        use tokio::io::AsyncWriteExt;

        let json = serde_json::to_vec(message)?;
        let length = json.len() as u32;

        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&json).await?;
        stream.flush().await?;

        Ok(())
    }
}

impl Default for PluginProtocol {
    fn default() -> Self {
        Self::new()
    }
}
