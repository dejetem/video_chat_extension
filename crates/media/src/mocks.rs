use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SfuConnection {
    async fn send_message(&self, msg: &str) -> Result<()>;
    async fn receive_message(&self) -> Result<String>;
}

pub struct MockSfu {
    pub last_sent: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl MockSfu {
    pub fn new() -> Self {
        Self {
            last_sent: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockSfu {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SfuConnection for MockSfu {
    async fn send_message(&self, msg: &str) -> Result<()> {
        let mut sent = self.last_sent.lock().await;
        sent.push(msg.to_string());
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        Ok("pong".to_string())
    }
}
