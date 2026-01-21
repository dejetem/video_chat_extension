#[cfg(test)]
mod tests_internal {
    #[cfg(target_arch = "wasm32")]
    use crate::SfuClient;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    #[cfg(target_arch = "wasm32")]
    use video_chat_signaling::stun_config::StunConfig;

    #[tokio::test]
    #[cfg(target_arch = "wasm32")]
    async fn test_sfu_client_initialization() {
        let stun = StunConfig {
            urls: vec!["stun:stun.l.google.com:19302".into()],
        };
        let client = SfuClient::new("test_room".into(), &stun).unwrap();
        assert_eq!(client.room_id, "test_room");
    }

    #[tokio::test]
    async fn test_reconnection_logic() {
        use crate::reconnection::ReconnectionManager;
        let manager = ReconnectionManager::new(3, Duration::from_millis(10));

        let count = Arc::new(Mutex::new(0));
        let result = manager
            .attempt_reconnect(|| {
                let count = count.clone();
                async move {
                    let mut c = count.lock().await;
                    *c += 1;
                    if *c < 3 {
                        Err(video_chat_core::error::Error::Internal("fail".into()))
                    } else {
                        Ok(())
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(*count.lock().await, 3);
    }
}
