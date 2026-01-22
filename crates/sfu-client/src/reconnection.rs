use std::time::Duration;
use video_chat_core::error::Result;

pub struct ReconnectionManager {
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl ReconnectionManager {
    pub fn new(max_retries: u32, retry_delay: Duration) -> Self {
        Self {
            max_retries,
            retry_delay,
        }
    }

    pub async fn attempt_reconnect<F, Fut>(&self, mut f: F) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut retries = 0;
        while retries < self.max_retries {
            match f().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    log::warn!("Reconnection attempt {} failed: {}", retries + 1, e);
                    retries += 1;
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
        Err(video_chat_core::error::Error::Internal(
            "Max reconnection retries exceeded".into(),
        ))
    }
}
